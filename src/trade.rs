use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    MeteoraClient, MeteoraError,
    global::METEORA_PROGRAM_ID,
    pool::PoolManager,
    types::{PoolInfo, SwapSimulation, TradeParams, TradeQuote},
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction::Message;
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

/// Main trade execution handler for Meteora DEX
pub struct Trade {
    client: Arc<MeteoraClient>,
    pool_manager: PoolManager,
    simulation_cache: HashMap<Pubkey, SwapSimulation>,
}

impl Trade {
    /// Creates a new Trade instance
    pub fn new(client: Arc<MeteoraClient>) -> Self {
        let pool_manager = PoolManager::new(client.clone());
        Self {
            client,
            pool_manager,
            simulation_cache: HashMap::new(),
        }
    }

    /// Gets a validated trade quote with comprehensive checks
    ///
    /// # Example
    /// ```
    /// let trade = Trade::new(client);
    /// let params = TradeParams {
    ///     input_mint: usdc_mint,
    ///     output_mint: sol_mint,
    ///     amount_in: 100_000_000, // 100 USDC
    ///     slippage_bps: 100, // 1%
    ///     user: user_pubkey,
    /// };
    /// let quote = trade.get_quote_with_validation(&params).await?;
    /// ```
    pub async fn get_quote_with_validation(
        &self,
        params: &TradeParams,
    ) -> Result<TradeQuote, MeteoraError> {
        self.validate_trade_params(params).await?;
        let pools = self
            .find_best_route(&params.input_mint, &params.output_mint)
            .await?;
        if pools.is_empty() {
            return Err(MeteoraError::NoLiquidityPoolFound);
        }
        let best_pool = self.select_best_pool(&pools).await?;
        let pool_info = self.pool_manager.get_pool_info(&best_pool).await?;
        let amount_out =
            self.calculate_swap_output(params.amount_in, &pool_info, &params.input_mint)?;
        let price_impact =
            self.calculate_price_impact(params.amount_in, &pool_info, &params.input_mint)?;
        if price_impact > params.slippage_bps as f64 / 100.0 {
            return Err(MeteoraError::SlippageExceeded);
        }
        let min_amount_out = amount_out * (10000 - params.slippage_bps as u64) / 10000;
        let fee_amount = params.amount_in * pool_info.trade_fee_bps / 10000;
        Ok(TradeQuote {
            amount_out,
            min_amount_out,
            price_impact,
            fee_amount,
            route: vec![best_pool],
        })
    }

    /// Executes a swap with comprehensive safety checks
    ///
    /// # Example
    /// ```
    /// let signature = trade.execute_swap_safe(&params, &user_keypair).await?;
    /// println!("Swap executed: {}", signature);
    /// ```
    pub async fn execute_swap_safe(
        &self,
        params: &TradeParams,
        user_keypair: &Keypair,
    ) -> Result<String, MeteoraError> {
        let quote = self.get_quote_with_validation(params).await?;
        let simulation = self.simulate_swap(params, &quote).await?;
        if !simulation.success {
            return Err(MeteoraError::TransactionFailed(
                "Simulation failed".to_string(),
            ));
        }
        if simulation.actual_output < quote.min_amount_out {
            return Err(MeteoraError::SlippageExceeded);
        }
        self.check_user_balance(&params.user, &params.input_mint, params.amount_in)
            .await?;
        let fee_estimate = self.estimate_transaction_fees().await?;
        let instructions = self.build_swap_instructions(params, &quote).await?;
        let signature = self
            .send_transaction(&instructions, user_keypair, fee_estimate)
            .await?;
        self.confirm_transaction_with_timeout(&signature, 30)
            .await?;
        Ok(signature)
    }

    async fn validate_trade_params(&self, params: &TradeParams) -> Result<(), MeteoraError> {
        if params.amount_in == 0 {
            return Err(MeteoraError::InvalidInput(
                "Amount cannot be zero".to_string(),
            ));
        }
        if params.slippage_bps > 5000 {
            // Maximum 50% slippage
            return Err(MeteoraError::InvalidInput("Slippage too high".to_string()));
        }
        if params.input_mint == params.output_mint {
            return Err(MeteoraError::InvalidInput(
                "Cannot swap same token".to_string(),
            ));
        }
        Ok(())
    }

    async fn find_best_route(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<Vec<Pubkey>, MeteoraError> {
        let pools = self
            .pool_manager
            .find_pools_by_tokens(input_mint, output_mint)
            .await?;
        let mut pool_liquidity = Vec::new();
        for pool in &pools {
            if let Ok(liquidity) = self.pool_manager.get_pool_liquidity(&pool.address).await {
                pool_liquidity.push((liquidity, pool.address));
            }
        }
        pool_liquidity.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(pool_liquidity.into_iter().map(|(_, addr)| addr).collect())
    }

    async fn select_best_pool(&self, pools: &[Pubkey]) -> Result<Pubkey, MeteoraError> {
        let mut best_pool = None;
        let mut best_score = 0.0;
        for pool_address in pools {
            if let Ok(pool_info) = self.pool_manager.get_pool_info(pool_address).await {
                let liquidity = pool_info.token_a_reserve_amount + pool_info.token_b_reserve_amount;
                let fee_score = 1.0 - (pool_info.trade_fee_bps as f64 / 10000.0);
                let score = liquidity as f64 * fee_score;
                if score > best_score {
                    best_score = score;
                    best_pool = Some(*pool_address);
                }
            }
        }
        best_pool.ok_or(MeteoraError::NoLiquidityPoolFound)
    }

    async fn simulate_swap(
        &self,
        params: &TradeParams,
        quote: &TradeQuote,
    ) -> Result<SwapSimulation, MeteoraError> {
        let instructions = self.build_swap_instructions(params, quote).await?;
        let recent_blockhash = self.get_recent_blockhash().await?;
        let message =
            Message::new_with_blockhash(&instructions, Some(&params.user), &recent_blockhash);
        // build transaction
        let transaction = Transaction::new_unsigned(message);
        // Simulate trading using RPC
        match self
            .client
            .solana
            .client_arc()
            .simulate_transaction(&transaction)
            .await
        {
            Ok(result) => {
                let simulation = SwapSimulation {
                    success: result.value.err.is_none(),
                    logs: result.value.logs.unwrap_or_default(),
                    units_consumed: result.value.units_consumed.unwrap_or(0),
                    price_impact: quote.price_impact,
                    actual_output: quote.amount_out,
                };
                Ok(simulation)
            }
            Err(e) => Err(MeteoraError::RpcError(e.to_string())),
        }
    }

    async fn check_user_balance(
        &self,
        user: &Pubkey,
        mint: &Pubkey,
        required_amount: u64,
    ) -> Result<(), MeteoraError> {
        let token_account = get_associated_token_address(user, mint);
        match self.client.get_account_data(&token_account).await {
            Ok(account_data) => {
                let account = spl_token::state::Account::unpack(&account_data)
                    .map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
                if account.amount < required_amount {
                    return Err(MeteoraError::InsufficientBalance);
                }
                Ok(())
            }
            Err(_) => Err(MeteoraError::AccountNotFound(
                "Token account not found".to_string(),
            )),
        }
    }

    async fn estimate_transaction_fees(&self) -> Result<u64, MeteoraError> {
        match self.client.solana.client_arc().get_latest_blockhash().await {
            Ok(blockhash) => {
                let message = Message::new_with_blockhash(&[], None, &blockhash);
                match self
                    .client
                    .solana
                    .client_arc()
                    .get_fee_for_message(&message)
                    .await
                {
                    Ok(fee) => Ok(fee),
                    Err(e) => {
                        log::warn!("Failed to get fee estimate: {}, using fallback", e);
                        let fallback_fee = 5000;
                        Ok(fallback_fee)
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to get blockhash for fee estimation: {}", e);
                Ok(10000)
            }
        }
    }

    async fn send_transaction(
        &self,
        instructions: &[Instruction],
        user_keypair: &Keypair,
        fee_estimate: u64,
    ) -> Result<String, MeteoraError> {
        let message = Message::new_with_blockhash(
            instructions,
            Some(&user_keypair.pubkey()),
            &self.get_recent_blockhash().await?,
        );
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[user_keypair], self.get_recent_blockhash().await?);
        match self
            .client
            .solana
            .client_arc()
            .send_and_confirm_transaction(&transaction)
            .await
        {
            Ok(signature) => Ok(signature.to_string()),
            Err(e) => Err(MeteoraError::TransactionFailed(e.to_string())),
        }
    }

    async fn get_recent_blockhash(&self) -> Result<solana_sdk::hash::Hash, MeteoraError> {
        self.client
            .solana
            .client_arc()
            .get_latest_blockhash()
            .await
            .map_err(|e| MeteoraError::RpcError(e.to_string()))
    }

    async fn confirm_transaction_with_timeout(
        &self,
        signature: &str,
        timeout_seconds: u64,
    ) -> Result<bool, MeteoraError> {
        let signature = signature
            .parse()
            .map_err(|_| MeteoraError::InvalidInput("Invalid signature".to_string()))?;
        for _ in 0..timeout_seconds {
            match self
                .client
                .solana
                .client_arc()
                .get_signature_status(&signature)
                .await
            {
                Ok(Some(status)) => {
                    if status.err().is_none() {
                        return Ok(true);
                    } else {
                        return Ok(false);
                    }
                }
                _ => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        Err(MeteoraError::TransactionFailed(
            "Confirmation timeout".to_string(),
        ))
    }

    /// Gets a quick trade quote without extensive validation
    ///
    /// # Example
    /// ```
    /// let quote = trade.get_quote(&params).await?;
    /// println!("Expected output: {}", quote.amount_out);
    /// ```
    pub async fn get_quote(&self, params: &TradeParams) -> Result<TradeQuote, MeteoraError> {
        let pools = self
            .pool_manager
            .find_pools_by_tokens(&params.input_mint, &params.output_mint)
            .await?;
        if pools.is_empty() {
            return Err(MeteoraError::NoLiquidityPoolFound);
        }
        let pool_info = &pools[0];
        let amount_out =
            self.calculate_swap_output(params.amount_in, pool_info, &params.input_mint)?;
        let min_amount_out = amount_out * (10000 - params.slippage_bps as u64) / 10000;
        let price_impact =
            self.calculate_price_impact(params.amount_in, pool_info, &params.input_mint)?;
        Ok(TradeQuote {
            amount_out,
            min_amount_out,
            price_impact,
            fee_amount: params.amount_in * pool_info.trade_fee_bps / 10000,
            route: vec![pool_info.address],
        })
    }

    /// Calculates swap output amount based on pool reserves
    fn calculate_swap_output(
        &self,
        amount_in: u64,
        pool_info: &PoolInfo,
        input_mint: &Pubkey,
    ) -> Result<u64, MeteoraError> {
        let (input_reserve, output_reserve) = if *input_mint == pool_info.token_a_mint {
            (
                pool_info.token_a_reserve_amount,
                pool_info.token_b_reserve_amount,
            )
        } else {
            (
                pool_info.token_b_reserve_amount,
                pool_info.token_a_reserve_amount,
            )
        };
        let amount_in_with_fee = amount_in * (10000 - pool_info.trade_fee_bps) / 10000;
        let numerator = amount_in_with_fee * output_reserve;
        let denominator = input_reserve * 10000 + amount_in_with_fee;
        if denominator == 0 {
            return Err(MeteoraError::CalculationError(
                "Division by zero".to_string(),
            ));
        }
        Ok(numerator / denominator)
    }

    /// Calculates price impact of the swap
    fn calculate_price_impact(
        &self,
        amount_in: u64,
        pool_info: &PoolInfo,
        input_mint: &Pubkey,
    ) -> Result<f64, MeteoraError> {
        let input_reserve = if *input_mint == pool_info.token_a_mint {
            pool_info.token_a_reserve_amount
        } else {
            pool_info.token_b_reserve_amount
        };
        if input_reserve == 0 {
            return Ok(100.0);
        }
        let price_impact = (amount_in as f64) / (input_reserve as f64 + amount_in as f64) * 100.0;
        Ok(price_impact)
    }

    async fn build_swap_instructions(
        &self,
        params: &TradeParams,
        quote: &TradeQuote,
    ) -> Result<Vec<Instruction>, MeteoraError> {
        let pool_info = self.pool_manager.get_pool_info(&quote.route[0]).await?;
        let user_input_account =
            self.get_associated_token_address(&params.user, &params.input_mint);
        let user_output_account =
            self.get_associated_token_address(&params.user, &params.output_mint);
        let mut instructions = Vec::new();
        if let Err(_) = self.client.get_account_data(&user_output_account).await {
            instructions.push(
                self.create_associated_token_account_instruction(&params.user, &params.output_mint),
            );
        }
        let swap_instruction = self.build_meteora_swap_instruction(
            params,
            quote,
            &pool_info,
            &user_input_account,
            &user_output_account,
        )?;
        instructions.push(swap_instruction);
        Ok(instructions)
    }

    fn build_meteora_swap_instruction(
        &self,
        params: &TradeParams,
        quote: &TradeQuote,
        pool_info: &PoolInfo,
        user_input_account: &Pubkey,
        user_output_account: &Pubkey,
    ) -> Result<Instruction, MeteoraError> {
        let (input_reserve, output_reserve) = if params.input_mint == pool_info.token_a_mint {
            (&pool_info.token_a_reserve, &pool_info.token_b_reserve)
        } else {
            (&pool_info.token_b_reserve, &pool_info.token_a_reserve)
        };
        let accounts = vec![
            AccountMeta::new(pool_info.address, false),
            AccountMeta::new_readonly(self.get_pool_authority(&pool_info.address)?, false),
            AccountMeta::new(params.user, true),
            AccountMeta::new(*user_input_account, false),
            AccountMeta::new(*input_reserve, false),
            AccountMeta::new(*output_reserve, false),
            AccountMeta::new(*user_output_account, false),
            AccountMeta::new(pool_info.fee_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];
        let mut data = Vec::new();
        data.push(9);
        data.extend_from_slice(&params.amount_in.to_le_bytes());
        data.extend_from_slice(&quote.min_amount_out.to_le_bytes());
        Ok(Instruction {
            program_id: Pubkey::from_str(METEORA_PROGRAM_ID).unwrap(),
            accounts,
            data,
        })
    }

    fn get_pool_authority(&self, pool_address: &Pubkey) -> Result<Pubkey, MeteoraError> {
        let (authority, _bump) = Pubkey::find_program_address(
            &[b"amm", pool_address.as_ref()],
            &Pubkey::from_str(METEORA_PROGRAM_ID).unwrap(),
        );
        Ok(authority)
    }
    fn get_associated_token_address(&self, wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
        get_associated_token_address(wallet, mint)
    }
    fn create_associated_token_account_instruction(
        &self,
        wallet: &Pubkey,
        mint: &Pubkey,
    ) -> Instruction {
        create_associated_token_account(wallet, wallet, mint, &spl_token::id())
    }

    /// Builds a token approve instruction
    ///
    /// # Example
    /// ```
    /// let approve_ix = trade.build_approve_instruction(
    ///     &user_pubkey,
    ///     &token_account,
    ///     &delegate_pubkey,
    ///     100_000_000,
    /// )?;
    /// ```
    pub fn build_approve_instruction(
        &self,
        owner: &Pubkey,
        token_account: &Pubkey,
        delegate: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, MeteoraError> {
        let instruction = spl_token::instruction::approve(
            &spl_token::id(),
            token_account,
            delegate,
            owner,
            &[],
            amount,
        )
        .map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
        Ok(instruction)
    }

    /// Builds a token transfer instruction
    ///
    /// # Example
    /// ```
    /// let transfer_ix = trade.build_transfer_instruction(
    ///     &from_account,
    ///     &to_account,
    ///     &owner_pubkey,
    ///     50_000_000,
    /// )?;
    /// ```
    pub fn build_transfer_instruction(
        &self,
        from: &Pubkey,
        to: &Pubkey,
        owner: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, MeteoraError> {
        let instruction =
            spl_token::instruction::transfer(&spl_token::id(), from, to, owner, &[], amount)
                .map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
        Ok(instruction)
    }

    /// Confirms transaction status
    ///
    /// # Example
    /// ```
    /// let confirmed = trade.confirm_transaction(&signature).await?;
    /// if confirmed {
    ///     println!("Transaction confirmed!");
    /// }
    /// ```
    pub async fn confirm_transaction(&self, signature: &str) -> Result<bool, MeteoraError> {
        match self
            .client
            .solana
            .client_arc()
            .get_signature_statuses(&[signature.parse().unwrap()])
            .await
        {
            Ok(statuses) => {
                if let Some(status) = statuses.value.get(0).and_then(|s| s.as_ref()) {
                    Ok(status.err.is_none())
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(MeteoraError::RpcError(e.to_string())),
        }
    }
}

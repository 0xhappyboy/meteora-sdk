use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::global::METEORA_PROGRAM_ID;
use crate::types::PoolInfo;
use crate::{MeteoraClient, MeteoraError};
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::{Account, Mint};
use tokio::time::Instant;

struct PoolCache {
    pools: HashMap<Pubkey, (PoolInfo, Instant)>,
    all_pools: Vec<Pubkey>,
    last_update: Instant,
    cache_ttl: Duration,
}

/// Manages Meteora pools with caching capabilities
pub struct PoolManager {
    client: Arc<MeteoraClient>,
    cache: Arc<Mutex<PoolCache>>,
}

impl PoolManager {
    /// Creates a new PoolManager instance
    pub fn new(client: Arc<MeteoraClient>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(PoolCache {
                pools: HashMap::new(),
                all_pools: Vec::new(),
                last_update: Instant::now() - Duration::from_secs(3600),
                cache_ttl: Duration::from_secs(300),
            })),
        }
    }
    /// Retrieves all pool addresses with caching
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use meteora_client::{MeteoraClient, PoolManager};
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let pool_manager = PoolManager::new(client);
    /// let pools = pool_manager.find_all_pools_cached().await?;
    /// ```
    pub async fn find_all_pools_cached(&self) -> Result<Vec<Pubkey>, MeteoraError> {
        let mut cache = self.cache.lock().unwrap();
        if cache.last_update.elapsed() < cache.cache_ttl && !cache.all_pools.is_empty() {
            return Ok(cache.all_pools.clone());
        }
        let accounts = self
            .client
            .get_program_accounts(&Pubkey::from_str(METEORA_PROGRAM_ID).unwrap(), None)
            .await?;
        let pools: Vec<Pubkey> = accounts.into_iter().map(|(pubkey, _)| pubkey).collect();
        cache.all_pools = pools.clone();
        cache.last_update = Instant::now();
        Ok(pools)
    }

    /// Retrieves pool information with caching
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use solana_sdk::pubkey::Pubkey;
    /// use meteora_client::{MeteoraClient, PoolManager};
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let pool_manager = PoolManager::new(client);
    /// let pool_address = Pubkey::new_unique();
    /// let pool_info = pool_manager.get_pool_info_cached(&pool_address).await?;
    /// ```
    pub async fn get_pool_info_cached(
        &self,
        pool_address: &Pubkey,
    ) -> Result<PoolInfo, MeteoraError> {
        let mut cache = self.cache.lock().unwrap();
        if let Some((cached_info, timestamp)) = cache.pools.get(pool_address) {
            if timestamp.elapsed() < cache.cache_ttl {
                return Ok(cached_info.clone());
            }
        }
        let pool_info = self.get_pool_info(pool_address).await?;
        cache
            .pools
            .insert(*pool_address, (pool_info.clone(), Instant::now()));
        Ok(pool_info)
    }

    /// Retrieves pool information directly from RPC
    pub async fn get_pool_info(&self, pool_address: &Pubkey) -> Result<PoolInfo, MeteoraError> {
        let pool_data = self.client.get_account_data(pool_address).await?;
        if pool_data.len() < 300 {
            return Err(MeteoraError::InvalidPoolData);
        }
        let token_a_mint = Pubkey::new_from_array(
            pool_data[8..40]
                .try_into()
                .map_err(|_| MeteoraError::InvalidPoolData)?,
        );
        let token_b_mint = Pubkey::new_from_array(
            pool_data[40..72]
                .try_into()
                .map_err(|_| MeteoraError::InvalidPoolData)?,
        );
        let token_a_reserve = Pubkey::new_from_array(
            pool_data[72..104]
                .try_into()
                .map_err(|_| MeteoraError::InvalidPoolData)?,
        );
        let token_b_reserve = Pubkey::new_from_array(
            pool_data[104..136]
                .try_into()
                .map_err(|_| MeteoraError::InvalidPoolData)?,
        );
        let lp_mint = Pubkey::new_from_array(
            pool_data[136..168]
                .try_into()
                .map_err(|_| MeteoraError::InvalidPoolData)?,
        );
        let fee_account = Pubkey::new_from_array(
            pool_data[168..200]
                .try_into()
                .map_err(|_| MeteoraError::InvalidPoolData)?,
        );
        let token_a_decimals = self.get_token_decimals(&token_a_mint).await?;
        let token_b_decimals = self.get_token_decimals(&token_b_mint).await?;
        let token_a_reserve_amount = self.get_token_balance(&token_a_reserve).await?;
        let token_b_reserve_amount = self.get_token_balance(&token_b_reserve).await?;
        let lp_supply = self.get_token_supply(&lp_mint).await?;
        Ok(PoolInfo {
            address: *pool_address,
            token_a_mint,
            token_b_mint,
            token_a_reserve,
            token_b_reserve,
            lp_mint,
            fee_account,
            trade_fee_bps: 30, // Meteora default fee 0.3%
            token_a_decimals,
            token_b_decimals,
            token_a_reserve_amount,
            token_b_reserve_amount,
            lp_supply,
        })
    }

    /// Finds pools that contain the specified token pair
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use solana_sdk::pubkey::Pubkey;
    /// use meteora_client::{MeteoraClient, PoolManager};
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let pool_manager = PoolManager::new(client);
    /// let token_a = Pubkey::new_unique();
    /// let token_b = Pubkey::new_unique();
    /// let pools = pool_manager.find_pools_by_tokens(&token_a, &token_b).await?;
    /// ```
    pub async fn find_pools_by_tokens(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
    ) -> Result<Vec<PoolInfo>, MeteoraError> {
        let all_pools = self.find_all_pools().await?;
        let mut matching_pools = Vec::new();
        for pool_address in all_pools {
            if let Ok(pool_info) = self.get_pool_info(&pool_address).await {
                if (pool_info.token_a_mint == *token_a && pool_info.token_b_mint == *token_b)
                    || (pool_info.token_a_mint == *token_b && pool_info.token_b_mint == *token_a)
                {
                    matching_pools.push(pool_info);
                }
            }
        }
        Ok(matching_pools)
    }

    /// Retrieves all pool addresses without caching
    pub async fn find_all_pools(&self) -> Result<Vec<Pubkey>, MeteoraError> {
        let accounts = self
            .client
            .get_program_accounts(&Pubkey::from_str(METEORA_PROGRAM_ID).unwrap(), None)
            .await?;
        Ok(accounts.into_iter().map(|(pubkey, _)| pubkey).collect())
    }

    /// Finds all pools that contain the specified token
    pub async fn find_token_pools(&self, token_mint: &Pubkey) -> Result<Vec<Pubkey>, MeteoraError> {
        let all_pools = self.find_all_pools().await?;
        let mut token_pools = Vec::new();
        for pool_address in all_pools {
            if let Ok(pool_info) = self.get_pool_info(&pool_address).await {
                if pool_info.token_a_mint == *token_mint || pool_info.token_b_mint == *token_mint {
                    token_pools.push(pool_address);
                }
            }
        }
        Ok(token_pools)
    }

    /// Calculates total liquidity for a pool
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use solana_sdk::pubkey::Pubkey;
    /// use meteora_client::{MeteoraClient, PoolManager};
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let pool_manager = PoolManager::new(client);
    /// let pool_address = Pubkey::new_unique();
    /// let liquidity = pool_manager.get_pool_liquidity(&pool_address).await?;
    /// ```
    pub async fn get_pool_liquidity(&self, pool_address: &Pubkey) -> Result<u64, MeteoraError> {
        let pool_info = self.get_pool_info(pool_address).await?;
        let liquidity = pool_info.token_a_reserve_amount + pool_info.token_b_reserve_amount;
        Ok(liquidity)
    }

    async fn get_token_balance(&self, token_account: &Pubkey) -> Result<u64, MeteoraError> {
        let account_data = self.client.get_account_data(token_account).await?;
        let token_account = Account::unpack(&account_data)
            .map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
        Ok(token_account.amount)
    }

    async fn get_token_decimals(&self, mint: &Pubkey) -> Result<u8, MeteoraError> {
        let account_data = self.client.get_account_data(mint).await?;
        let token_mint = Mint::unpack(&account_data)
            .map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
        Ok(token_mint.decimals)
    }

    async fn get_token_supply(&self, mint: &Pubkey) -> Result<u64, MeteoraError> {
        let account_data = self.client.get_account_data(mint).await?;
        let token_mint = Mint::unpack(&account_data)
            .map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
        Ok(token_mint.supply)
    }
}

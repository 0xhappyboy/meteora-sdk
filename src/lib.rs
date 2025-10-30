use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_commitment_config::CommitmentConfig;
use solana_network_sdk::Solana;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::sync::Arc;

use crate::types::MeteoraError;
use solana_network_sdk::types::Mode;
pub mod events;
pub mod global;
pub mod pool;
pub mod price;
pub mod token;
pub mod trade;
pub mod types;

/// A client for interacting with the Meteora protocol on Solana
/// Provides methods to fetch account data, program accounts, and SPL token accounts
pub struct MeteoraClient {
    pub solana: Arc<Solana>,
    pub commitment: CommitmentConfig,
}

impl MeteoraClient {
    /// Creates a new MeteoraClient with the default confirmed commitment
    ///
    /// # Params
    /// mode - Solana Network Mode
    ///
    /// # Example
    /// ```
    /// use meteora_client::MeteoraClient;
    ///
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// ```
    pub fn new(mode: Mode) -> Result<Self, MeteoraError> {
        Ok(Self {
            solana: Arc::new(
                Solana::new(mode).map_err(|e| MeteoraError::Error(format!("{:?}", e)))?,
            ),
            commitment: CommitmentConfig::confirmed(),
        })
    }

    /// Creates a new MeteoraClient with a custom commitment level
    ///
    /// # Params
    /// mode - Solana Network Mode
    /// commitment - The commitment level for queries
    ///
    /// # Example
    /// ```
    /// use meteora_client::MeteoraClient;
    /// use solana_commitment_config::CommitmentConfig;
    ///
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// ```
    pub fn new_with_commitment(
        mode: Mode,
        commitment: CommitmentConfig,
    ) -> Result<Self, MeteoraError> {
        Ok(Self {
            solana: Arc::new(
                Solana::new(mode).map_err(|e| MeteoraError::Error(format!("{:?}", e)))?,
            ),
            commitment: CommitmentConfig::confirmed(),
        })
    }

    /// Fetches the raw account data for a given address
    ///
    /// # Params
    /// address - The Pubkey of the account to fetch
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey;
    /// use meteora_client::MeteoraClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let account_pubkey = pubkey!("So11111111111111111111111111111111111111112");
    /// let account_data = client.get_account_data(&account_pubkey)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_account_data(&self, address: &Pubkey) -> Result<Vec<u8>, MeteoraError> {
        match self
            .solana
            .client
            .clone()
            .unwrap()
            .get_account_with_commitment(address, self.commitment)
            .await
        {
            Ok(account) => {
                if let Some(account) = account.value {
                    Ok(account.data)
                } else {
                    Err(MeteoraError::AccountNotFound(format!(
                        "Account {} not found",
                        address
                    )))
                }
            }
            Err(e) => Err(MeteoraError::RpcError(e.to_string())),
        }
    }

    /// Fetches raw account data for multiple addresses in a single request
    ///
    /// # Params
    /// addresses - Slice of Pubkeys to fetch
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey;
    /// use meteora_client::MeteoraClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let addresses = vec![
    ///     pubkey!("So11111111111111111111111111111111111111112"),
    ///     pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    /// ];
    /// let accounts_data = client.get_multiple_accounts_data(&addresses)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_multiple_accounts_data(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Vec<u8>>, MeteoraError> {
        match self
            .solana
            .client
            .clone()
            .unwrap()
            .get_multiple_accounts_with_commitment(addresses, self.commitment)
            .await
        {
            Ok(accounts) => {
                let mut results = Vec::new();
                for account in accounts.value {
                    if let Some(account) = account {
                        results.push(account.data);
                    } else {
                        results.push(Vec::new());
                    }
                }
                Ok(results)
            }
            Err(e) => Err(MeteoraError::RpcError(e.to_string())),
        }
    }

    /// Fetches all accounts owned by a program with optional filters
    ///
    /// # Params
    /// program_id - The program ID to query
    /// filters - Optional filters to apply to the query
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey;
    /// use solana_client::rpc_filter::RpcFilterType;
    /// use meteora_client::MeteoraClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let program_id = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    /// let filters = vec![RpcFilterType::DataSize(165)];
    /// let program_accounts = client.get_program_accounts(&program_id, Some(filters))?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<RpcFilterType>>,
    ) -> Result<Vec<(Pubkey, Account)>, MeteoraError> {
        let config = RpcProgramAccountsConfig {
            filters: Some(filters.unwrap_or_default()),
            account_config: RpcAccountInfoConfig {
                commitment: Some(self.commitment),
                encoding: None,
                data_slice: None,
                min_context_slot: None,
            },
            with_context: None,
            sort_results: None,
        };
        match self
            .solana
            .client
            .clone()
            .unwrap()
            .get_program_accounts_with_config(program_id, config)
            .await
        {
            Ok(accounts) => Ok(accounts),
            Err(e) => Err(MeteoraError::RpcError(e.to_string())),
        }
    }

    /// Fetches all SPL token accounts for a specific mint address
    ///
    /// # Params
    /// mint - The mint address of the token
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey;
    /// use meteora_client::MeteoraClient;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    /// let token_accounts = client.get_spl_token_accounts_by_mint(&usdc_mint)?;
    /// Ok(())
    /// }
    /// ```
    pub async fn get_spl_token_accounts_by_mint(
        &self,
        mint: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, MeteoraError> {
        let filters = vec![
            RpcFilterType::DataSize(165),
            RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &mint.to_bytes())),
        ];
        self.get_program_accounts(&spl_token::id(), Some(filters))
            .await
    }
}

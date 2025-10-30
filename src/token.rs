use std::str::FromStr;

use crate::global::METAPLEX_PROGRAM_ID;
use crate::types::{TokenInfo, TokenMetadata};
use crate::{MeteoraClient, MeteoraError};
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Mint;

/// Manages token-related operations including fetching token information,
/// holder counts, and metadata.
pub struct TokenManager {
    client: MeteoraClient,
}

impl TokenManager {
    /// Creates a new TokenManager instance.
    ///
    /// # Example
    /// ```
    /// use meteora_client::MeteoraClient;
    /// use meteora_client::token::TokenManager;
    ///
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let token_manager = TokenManager::new(client);
    /// ```
    pub fn new(client: MeteoraClient) -> Self {
        Self { client }
    }

    /// Fetches comprehensive information about a token.
    ///
    /// # Params
    /// mint - The mint address of the token
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey::Pubkey;
    /// use meteora_client::token::TokenManager;
    /// use meteora_client::MeteoraClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let token_manager = TokenManager::new(client);
    /// let usdc_mint = Pubkey::new_from_array([/* USDC mint address */]);
    /// match token_manager.get_token_info(&usdc_mint).await {
    ///     Ok(token_info) => println!("Token decimals: {}", token_info.decimals),
    ///     Err(e) => eprintln!("Error fetching token info: {}", e),
    /// }
    /// }
    /// ```
    pub async fn get_token_info(&self, mint: &Pubkey) -> Result<TokenInfo, MeteoraError> {
        let mint_account_data = self.client.get_account_data(mint).await?;
        let (decimals, supply) = self.parse_mint_account(&mint_account_data)?;
        let holder_count = self.get_holder_count(mint).await?;
        let metadata = self.get_token_metadata(mint).await.ok();
        Ok(TokenInfo {
            mint: *mint,
            decimals,
            supply,
            holder_count,
            metadata,
        })
    }

    /// Counts the number of token holders for a given mint.
    ///
    /// # Params
    /// mint - The mint address of the token
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey::Pubkey;
    /// use meteora_client::token::TokenManager;
    /// use meteora_client::MeteoraClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let token_manager = TokenManager::new(client);
    /// let mint = Pubkey::new_from_array([/* token mint address */]);
    /// match token_manager.get_holder_count(&mint).await {
    ///     Ok(count) => println!("Token has {} holders", count),
    ///     Err(e) => eprintln!("Error fetching holder count: {}", e),
    /// }
    /// }
    /// ```
    pub async fn get_holder_count(&self, mint: &Pubkey) -> Result<u64, MeteoraError> {
        let accounts = self.client.get_spl_token_accounts_by_mint(mint).await?;
        Ok(accounts.len() as u64)
    }

    /// Fetches token metadata from the Metaplex metadata account.
    ///
    /// # Params
    /// mint - The mint address of the token
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey::Pubkey;
    /// use meteora_client::token::TokenManager;
    /// use meteora_client::MeteoraClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    /// let client = MeteoraClient::new(solana_network_sdk::types::Mode::MAIN);
    /// let token_manager = TokenManager::new(client);
    /// let mint = Pubkey::new_from_array([/* token mint address */]);
    /// match token_manager.get_token_metadata(&mint).await {
    ///     Ok(metadata) => println!("Token name: {}", metadata.name),
    ///     Err(e) => eprintln!("Error fetching metadata: {}", e),
    /// }
    /// }
    /// ```
    pub async fn get_token_metadata(&self, mint: &Pubkey) -> Result<TokenMetadata, MeteoraError> {
        let metadata_address = self.get_metadata_account(mint);
        match self.client.get_account_data(&metadata_address).await {
            Ok(data) => self.parse_metadata_account(&data),
            Err(_) => Err(MeteoraError::AccountNotFound(
                "Token metadata not found".to_string(),
            )),
        }
    }

    fn parse_mint_account(&self, data: &[u8]) -> Result<(u8, u64), MeteoraError> {
        let token_mint =
            Mint::unpack(data).map_err(|e| MeteoraError::DeserializationError(e.to_string()))?;
        Ok((token_mint.decimals, token_mint.supply))
    }

    fn get_metadata_account(&self, mint: &Pubkey) -> Pubkey {
        let metaplex_program_id =
            Pubkey::from_str(METAPLEX_PROGRAM_ID).expect("Failed to parse Metaplex program ID");
        let seeds = &[b"metadata", metaplex_program_id.as_ref(), mint.as_ref()];
        Pubkey::find_program_address(seeds, &metaplex_program_id).0
    }

    fn parse_metadata_account(&self, data: &[u8]) -> Result<TokenMetadata, MeteoraError> {
        if data.len() < 100 {
            return Err(MeteoraError::InvalidAccountData);
        }
        let name_start = 1 + 32 + 32; // key + update auth + mint
        let name_length = data[name_start] as usize;
        let name_end = name_start + 1 + name_length;
        if name_end >= data.len() {
            return Err(MeteoraError::InvalidAccountData);
        }
        let name = String::from_utf8_lossy(&data[name_start + 1..name_end]).to_string();
        let symbol_start = name_end + 4; // +4 for URI length prefix
        let symbol_length = data[symbol_start] as usize;
        let symbol_end = symbol_start + 1 + symbol_length;
        if symbol_end >= data.len() {
            return Err(MeteoraError::InvalidAccountData);
        }
        let symbol = String::from_utf8_lossy(&data[symbol_start + 1..symbol_end]).to_string();
        let uri_start = symbol_end + 4; // +4 for URI length prefix
        let uri_length = data[uri_start] as usize;
        let uri_end = uri_start + 1 + uri_length;
        if uri_end > data.len() {
            return Err(MeteoraError::InvalidAccountData);
        }
        let uri = String::from_utf8_lossy(&data[uri_start + 1..uri_end]).to_string();
        Ok(TokenMetadata { name, symbol, uri })
    }
}

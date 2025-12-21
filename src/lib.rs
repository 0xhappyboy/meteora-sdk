use std::{str::FromStr, sync::Arc};

use solana_network_client::SolanaClient;
use solana_sdk::pubkey::Pubkey;
use solana_transaction::Address;

use crate::{
    dbc::{DynamicBondingCurvePool, DynamicBondingCurvePoolData},
    liquidity::{
        dlmmpool::{DLMMLiquidityPool, DLMMLiquidityPoolData},
        dynpool::{DAMMLiquidityPool, DAMMLiquidityPoolData},
        dynv2pool::{DAMMV2LiquidityPool, DAMMV2LiquidityPoolData},
    },
};

pub mod dbc;
pub mod liquidity;

/// A client for interacting with the Meteora protocol on Solana
/// Provides methods to fetch account data, program accounts, and SPL token accounts
pub struct Meteora {
    pub solana_client: Arc<SolanaClient>,
}

impl Meteora {
    /// crreate raydium
    /// Example
    /// ```rust
    /// let sol = Solana::new(solana_network_sdk::types::Mode::MAIN);
    /// let raydium = Meteora::new(Arc::new(sol));
    /// ```
    pub fn new(solana_client: Arc<SolanaClient>) -> Self {
        Self { solana_client }
    }

    /// get dynv2 meteora liquidity pool
    /// Example
    /// ```rust
    /// let sol = Solana::new(solana_network_sdk::types::Mode::MAIN);
    /// let meteora = Meteora::new(Arc::new(sol));
    /// // 5gB4NPgFB3MHFHSeKN4sbaY6t9MB8ikCe9HyiKYid4Td SOL-USDC pool
    /// let pool_data = meteora.get_liquidity_pool_dynv2("5gB4NPgFB3MHFHSeKN4sbaY6t9MB8ikCe9HyiKYid4Td").await;
    /// ```
    pub async fn get_liquidity_pool_dynv2(
        &self,
        address: &str,
    ) -> Result<DAMMV2LiquidityPoolData, String> {
        let v = self
            .solana_client
            .client_arc()
            .get_account_data(
                &Pubkey::from_str(address)
                    .map_err(|e| format!("{:?}", e))
                    .unwrap(),
            )
            .await
            .map_err(|e| format!("{:?}", e))?;
        let pool =
            DAMMV2LiquidityPool::get_liquidity_pool_info(&v).map_err(|e| format!("{:?}", e))?;
        Ok(pool)
    }

    /// get dynv2 meteora liquidity pool
    /// Example
    /// ```rust
    /// let sol = Solana::new(solana_network_sdk::types::Mode::MAIN);
    /// let meteora = Meteora::new(Arc::new(sol));
    /// // DqAfrGV2GBxpGRsq6Xk1z9ojRncqgLeeVPaKg5bCc24Z SOL-USDC pool
    /// let pool_data = meteora.get_liquidity_pool_dyn("DqAfrGV2GBxpGRsq6Xk1z9ojRncqgLeeVPaKg5bCc24Z").await;
    /// ```
    pub async fn get_liquidity_pool_dyn(
        &self,
        address: &str,
    ) -> Result<DAMMLiquidityPoolData, String> {
        let v = self
            .solana_client
            .client_arc()
            .get_account_data(
                &Pubkey::from_str(address)
                    .map_err(|e| format!("{:?}", e))
                    .unwrap(),
            )
            .await
            .map_err(|e| format!("{:?}", e))?;
        let pool =
            DAMMLiquidityPool::get_liquidity_pool_info(&v).map_err(|e| format!("{:?}", e))?;
        Ok(pool)
    }

    /// get dynv2 meteora liquidity pool
    /// Example
    /// ```rust
    /// let sol = Solana::new(solana_network_sdk::types::Mode::MAIN);
    /// let meteora = Meteora::new(Arc::new(sol));
    /// // BjxkogRUDnb72MSBTfsyuq54yntqxyVKozK9WywMszvZ SOL-USDC pool
    /// let pool_data = meteora.get_liquidity_pool_dlmm("BjxkogRUDnb72MSBTfsyuq54yntqxyVKozK9WywMszvZ").await;
    /// ```
    pub async fn get_liquidity_pool_dlmm(
        &self,
        address: &str,
    ) -> Result<DLMMLiquidityPoolData, String> {
        let v = self
            .solana_client
            .client_arc()
            .get_account_data(
                &Pubkey::from_str(address)
                    .map_err(|e| format!("{:?}", e))
                    .unwrap(),
            )
            .await
            .map_err(|e| format!("{:?}", e))?;
        let pool =
            DLMMLiquidityPool::get_liquidity_pool_info(&v).map_err(|e| format!("{:?}", e))?;
        Ok(pool)
    }

    /// get dynv2 meteora liquidity pool
    /// Example
    /// ```rust
    /// let sol = Solana::new(solana_network_sdk::types::Mode::MAIN);
    /// let meteora = Meteora::new(Arc::new(sol));
    /// // BjxkogRUDnb72MSBTfsyuq54yntqxyVKozK9WywMszvZ SOL-USDC pool
    /// let pool_data = meteora.get_liquidity_pool_dlmm("BjxkogRUDnb72MSBTfsyuq54yntqxyVKozK9WywMszvZ").await;
    /// ```
    pub async fn get_liquidity_pool_dbc(
        &self,
        address: &str,
    ) -> Result<DynamicBondingCurvePoolData, String> {
        let v = self
            .solana_client
            .client_arc()
            .get_account_data(
                &Pubkey::from_str(address)
                    .map_err(|e| format!("{:?}", e))
                    .unwrap(),
            )
            .await
            .map_err(|e| format!("{:?}", e))?;
        let pool =
            DynamicBondingCurvePool::get_dbc_pool_info(&v).map_err(|e| format!("{:?}", e))?;
        Ok(pool)
    }
}

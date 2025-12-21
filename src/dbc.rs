use std::mem;

use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

pub const VIRTUAL_POOL_DATA_SIZE: usize = 416;
const DISCRIMINATOR_LEN: usize = 8;

unsafe impl Pod for DynamicBondingCurvePool {}
unsafe impl Zeroable for DynamicBondingCurvePool {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DynamicBondingCurvePool {
    /// volatility tracker
    pub volatility_tracker: [u8; 64],
    /// config key (32 bytes)
    pub config: [u8; 32],
    /// creator (32 bytes)
    pub creator: [u8; 32],
    /// base mint (32 bytes)
    pub base_mint: [u8; 32],
    /// base vault (32 bytes)
    pub base_vault: [u8; 32],
    /// quote vault (32 bytes)
    pub quote_vault: [u8; 32],
    /// base reserve (8 bytes)
    pub base_reserve: [u8; 8],
    /// quote reserve (8 bytes)
    pub quote_reserve: [u8; 8],
    /// protocol base fee (8 bytes)
    pub protocol_base_fee: [u8; 8],
    /// protocol quote fee (8 bytes)
    pub protocol_quote_fee: [u8; 8],
    /// partner base fee (8 bytes)
    pub partner_base_fee: [u8; 8],
    /// partner quote fee (8 bytes)
    pub partner_quote_fee: [u8; 8],
    /// current price (16 bytes for u128)
    pub sqrt_price: [u8; 16],
    /// Activation point (8 bytes)
    pub activation_point: [u8; 8],
    /// pool type (1 byte)
    pub pool_type: u8,
    /// is migrated (1 byte)
    pub is_migrated: u8,
    /// is partner withdraw surplus (1 byte)
    pub is_partner_withdraw_surplus: u8,
    /// is protocol withdraw surplus (1 byte)
    pub is_protocol_withdraw_surplus: u8,
    /// migration progress (1 byte)
    pub migration_progress: u8,
    /// is withdraw leftover (1 byte)
    pub is_withdraw_leftover: u8,
    /// is creator withdraw surplus (1 byte)
    pub is_creator_withdraw_surplus: u8,
    /// migration fee withdraw status (1 byte)
    pub migration_fee_withdraw_status: u8,
    /// pool metrics (32 bytes for PoolMetrics)
    pub metrics: [u8; 32],
    /// The time curve is finished (8 bytes)
    pub finish_curve_timestamp: [u8; 8],
    /// creator base fee (8 bytes)
    pub creator_base_fee: [u8; 8],
    /// creator quote fee (8 bytes)
    pub creator_quote_fee: [u8; 8],
    /// creation fee bits (1 byte)
    pub creation_fee_bits: u8,
    /// padding (7 bytes)
    pub _padding_0: [u8; 7],
    /// Padding for further use (48 bytes = [u64; 6])
    pub _padding_1: [u8; 48],
}

#[derive(Debug, Clone)]
pub struct DynamicBondingCurvePoolData {
    // Token addresses
    pub config: Pubkey,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_reserve: u64,
    pub quote_reserve: u64,
    pub protocol_base_fee: u64,
    pub protocol_quote_fee: u64,
    pub partner_base_fee: u64,
    pub partner_quote_fee: u64,
    pub creator_base_fee: u64,
    pub creator_quote_fee: u64,
    pub sqrt_price: u128,
    pub activation_point: u64,
    pub finish_curve_timestamp: u64,
    pub pool_type: u8,
    pub is_migrated: bool,
    pub is_partner_withdraw_surplus: bool,
    pub is_protocol_withdraw_surplus: bool,
    pub migration_progress: u8,
    pub is_withdraw_leftover: bool,
    pub is_creator_withdraw_surplus: bool,
    pub migration_fee_withdraw_status: u8,
    pub total_protocol_base_fee: u64,
    pub total_protocol_quote_fee: u64,
    pub total_trading_base_fee: u64,
    pub total_trading_quote_fee: u64,
    pub creation_fee_bits: u8,
    pub has_creation_fee: bool,
    pub creation_fee_claimed: bool,
    pub volatility_tracker: VolatilityTrackerData,
}

/// Volatility tracker
#[derive(Debug, Clone)]
pub struct VolatilityTrackerData {
    pub last_update_timestamp: u64,
    pub padding: [u8; 8],
    pub sqrt_price_reference: u128,
    pub volatility_accumulator: u128,
    pub volatility_reference: u128,
}

/// Pool metrics structure
#[derive(Debug, Clone)]
pub struct PoolMetricsData {
    pub total_protocol_base_fee: u64,
    pub total_protocol_quote_fee: u64,
    pub total_trading_base_fee: u64,
    pub total_trading_quote_fee: u64,
}

impl DynamicBondingCurvePool {
    /// Parse Virtual Pool data from byte array
    pub fn get_dbc_pool_info(data: &[u8]) -> Result<DynamicBondingCurvePoolData, String> {
        let total_expected_size = DISCRIMINATOR_LEN + VIRTUAL_POOL_DATA_SIZE;
        if data.len() != total_expected_size {
            return Err(format!(
                "Virtual pool data size mismatch. Expected {} ({} data + {} discriminator), got {}",
                total_expected_size,
                VIRTUAL_POOL_DATA_SIZE,
                DISCRIMINATOR_LEN,
                data.len()
            ));
        }
        let pool_data = &data[DISCRIMINATOR_LEN..];
        let struct_size = mem::size_of::<DynamicBondingCurvePool>();
        if struct_size != VIRTUAL_POOL_DATA_SIZE {
            return Err(format!(
                "Struct size mismatch. DynamicBondingCurvePool  is {} bytes, but VIRTUAL_POOL_DATA_SIZE is {}",
                struct_size, VIRTUAL_POOL_DATA_SIZE
            ));
        }
        let pool: &DynamicBondingCurvePool = bytemuck::try_from_bytes(pool_data)
            .map_err(|e| format!("Failed to cast bytes to DynamicBondingCurvePool : {}", e))?;
        Self::parse_from_struct(pool)
    }

    fn parse_from_struct(
        pool: &DynamicBondingCurvePool,
    ) -> Result<DynamicBondingCurvePoolData, String> {
        // volatility tracker (64 bytes)
        let volatility_tracker = VolatilityTrackerData {
            last_update_timestamp: u64::from_le_bytes(
                pool.volatility_tracker[0..8].try_into().unwrap(),
            ),
            padding: pool.volatility_tracker[8..16].try_into().unwrap(),
            sqrt_price_reference: u128::from_le_bytes(
                pool.volatility_tracker[16..32].try_into().unwrap(),
            ),
            volatility_accumulator: u128::from_le_bytes(
                pool.volatility_tracker[32..48].try_into().unwrap(),
            ),
            volatility_reference: u128::from_le_bytes(
                pool.volatility_tracker[48..64].try_into().unwrap(),
            ),
        };
        let total_protocol_base_fee = u64::from_le_bytes(pool.metrics[0..8].try_into().unwrap());
        let total_protocol_quote_fee = u64::from_le_bytes(pool.metrics[8..16].try_into().unwrap());
        let total_trading_base_fee = u64::from_le_bytes(pool.metrics[16..24].try_into().unwrap());
        let total_trading_quote_fee = u64::from_le_bytes(pool.metrics[24..32].try_into().unwrap());
        let creation_fee_bits = pool.creation_fee_bits;
        let has_creation_fee = (creation_fee_bits & 0b01) != 0;
        let creation_fee_claimed = (creation_fee_bits & 0b10) != 0;
        Ok(DynamicBondingCurvePoolData {
            config: Pubkey::new_from_array(pool.config),
            creator: Pubkey::new_from_array(pool.creator),
            base_mint: Pubkey::new_from_array(pool.base_mint),
            base_vault: Pubkey::new_from_array(pool.base_vault),
            quote_vault: Pubkey::new_from_array(pool.quote_vault),
            base_reserve: u64::from_le_bytes(pool.base_reserve),
            quote_reserve: u64::from_le_bytes(pool.quote_reserve),
            protocol_base_fee: u64::from_le_bytes(pool.protocol_base_fee),
            protocol_quote_fee: u64::from_le_bytes(pool.protocol_quote_fee),
            partner_base_fee: u64::from_le_bytes(pool.partner_base_fee),
            partner_quote_fee: u64::from_le_bytes(pool.partner_quote_fee),
            creator_base_fee: u64::from_le_bytes(pool.creator_base_fee),
            creator_quote_fee: u64::from_le_bytes(pool.creator_quote_fee),
            sqrt_price: u128::from_le_bytes(pool.sqrt_price),
            activation_point: u64::from_le_bytes(pool.activation_point),
            finish_curve_timestamp: u64::from_le_bytes(pool.finish_curve_timestamp),
            pool_type: pool.pool_type,
            is_migrated: pool.is_migrated != 0,
            is_partner_withdraw_surplus: pool.is_partner_withdraw_surplus != 0,
            is_protocol_withdraw_surplus: pool.is_protocol_withdraw_surplus != 0,
            migration_progress: pool.migration_progress,
            is_withdraw_leftover: pool.is_withdraw_leftover != 0,
            is_creator_withdraw_surplus: pool.is_creator_withdraw_surplus != 0,
            migration_fee_withdraw_status: pool.migration_fee_withdraw_status,
            total_protocol_base_fee,
            total_protocol_quote_fee,
            total_trading_base_fee,
            total_trading_quote_fee,
            creation_fee_bits,
            has_creation_fee,
            creation_fee_claimed,
            volatility_tracker,
        })
    }

    // Getter methods for public keys
    pub fn config_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.config)
    }

    pub fn creator_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.creator)
    }

    pub fn base_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.base_mint)
    }

    pub fn base_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.base_vault)
    }

    pub fn quote_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.quote_vault)
    }
}

impl DynamicBondingCurvePoolData {
    /// Get pool type as string
    pub fn get_pool_type_str(&self) -> &'static str {
        match self.pool_type {
            0 => "SplToken",
            1 => "Token2022",
            _ => "Unknown",
        }
    }

    /// Get migration progress as string
    pub fn get_migration_progress_str(&self) -> &'static str {
        match self.migration_progress {
            0 => "PreBondingCurve",
            1 => "PostBondingCurve",
            2 => "LockedVesting",
            3 => "CreatedPool",
            _ => "Unknown",
        }
    }

    /// Get creation fee status
    pub fn has_creation_fee(&self) -> bool {
        self.has_creation_fee
    }

    /// Get creation fee claimed status
    pub fn creation_fee_claimed(&self) -> bool {
        self.creation_fee_claimed
    }

    /// Get migration fee withdraw eligibility
    pub fn eligible_to_withdraw_migration_fee(&self, mask: u8) -> bool {
        (self.migration_fee_withdraw_status & mask) == 0
    }

    /// Check if curve is complete
    pub fn is_curve_complete(&self, migration_threshold: u64) -> bool {
        self.quote_reserve >= migration_threshold
    }

    /// Get protocol and trading base fee total
    pub fn get_protocol_and_trading_base_fee(&self) -> u64 {
        self.partner_base_fee
            .saturating_add(self.protocol_base_fee)
            .saturating_add(self.creator_base_fee)
    }

    /// Get total surplus
    pub fn get_total_surplus(&self, migration_threshold: u64) -> Option<u64> {
        self.quote_reserve.checked_sub(migration_threshold)
    }

    /// Get price from sqrt price
    pub fn get_price(&self) -> f64 {
        let sqrt_price = self.sqrt_price as f64;
        let price = (sqrt_price * sqrt_price) / (2.0f64.powi(64));
        price
    }

    /// Get volatility accumulator as percentage
    pub fn get_volatility_accumulator_percentage(&self) -> f64 {
        (self.volatility_tracker.volatility_accumulator as f64) / 100.0
    }

    /// Get protocol fees
    pub fn get_protocol_fees(&self) -> (u64, u64) {
        (self.protocol_base_fee, self.protocol_quote_fee)
    }

    /// Get partner fees
    pub fn get_partner_fees(&self) -> (u64, u64) {
        (self.partner_base_fee, self.partner_quote_fee)
    }

    /// Get creator fees
    pub fn get_creator_fees(&self) -> (u64, u64) {
        (self.creator_base_fee, self.creator_quote_fee)
    }

    /// Get all reserves
    pub fn get_reserves(&self) -> (u64, u64) {
        (self.base_reserve, self.quote_reserve)
    }

    /// Check if pool is migrated
    pub fn is_migrated(&self) -> bool {
        self.is_migrated
    }

    /// Check if partner can withdraw surplus
    pub fn can_partner_withdraw_surplus(&self) -> bool {
        self.is_partner_withdraw_surplus
    }

    /// Check if protocol can withdraw surplus
    pub fn can_protocol_withdraw_surplus(&self) -> bool {
        self.is_protocol_withdraw_surplus
    }

    /// Check if creator can withdraw surplus
    pub fn can_creator_withdraw_surplus(&self) -> bool {
        self.is_creator_withdraw_surplus
    }

    /// Check if leftover can be withdrawn
    pub fn can_withdraw_leftover(&self) -> bool {
        self.is_withdraw_leftover
    }

    /// Get pool metrics
    pub fn get_metrics(&self) -> PoolMetricsData {
        PoolMetricsData {
            total_protocol_base_fee: self.total_protocol_base_fee,
            total_protocol_quote_fee: self.total_protocol_quote_fee,
            total_trading_base_fee: self.total_trading_base_fee,
            total_trading_quote_fee: self.total_trading_quote_fee,
        }
    }
}

/// Dynamic Bonding Curve Info structure
#[derive(Debug, Clone)]
pub struct DynamicBondingCurvePoolInfo {
    pub virtual_pool_data: DynamicBondingCurvePoolData,
    pub additional_info: Option<String>,
}

impl DynamicBondingCurvePoolInfo {
    /// Create new Dynamic Bonding Curve Info
    pub fn new(virtual_pool_data: DynamicBondingCurvePoolData) -> Self {
        Self {
            virtual_pool_data,
            additional_info: None,
        }
    }

    /// Set additional info
    pub fn set_additional_info(&mut self, info: String) {
        self.additional_info = Some(info);
    }

    /// Get summary of the bonding curve
    pub fn get_summary(&self) -> String {
        format!(
            "Virtual Pool: {} -> {}, Reserves: {} base / {} quote, Price: {:.6}, Status: {}",
            self.virtual_pool_data.base_mint,
            self.virtual_pool_data.quote_vault,
            self.virtual_pool_data.base_reserve,
            self.virtual_pool_data.quote_reserve,
            self.virtual_pool_data.get_price(),
            self.virtual_pool_data.get_migration_progress_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Meteora;
    use solana_network_client::{Mode, SolanaClient};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_dbc_pool() {
        let solana_client = SolanaClient::new(Mode::MAIN).unwrap();
        let meteora = Meteora::new(Arc::new(solana_client));
        let pool_data = meteora
            .get_liquidity_pool_dbc("FDT74DRFm6d2zig9ZT1ABT3NJMPooWXs1tCMvNzvd2jV")
            .await
            .unwrap();
        println!("Pool Data: {:?}", pool_data);
    }
}

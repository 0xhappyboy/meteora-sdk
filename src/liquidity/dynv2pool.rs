use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

/// DAMMV2 liquidity pool data size (1104 bytes from the official layout)
pub const DAMMV2_LIQUIDITY_POOL_DATA_SIZE: usize = 1104;
const DISCRIMINATOR_LEN: usize = 8;

unsafe impl Pod for DAMMV2LiquidityPool {}
unsafe impl Zeroable for DAMMV2LiquidityPool {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DAMMV2LiquidityPool {
    // 0-159: Pool fee structure (160 bytes)
    pub pool_fees: [u8; 160],
    // 160-191: Token A mint address
    pub token_a_mint: [u8; 32],
    // 192-223: Token B mint address
    pub token_b_mint: [u8; 32],
    // 224-255: Token A vault address
    pub token_a_vault: [u8; 32],
    // 256-287: Token B vault address
    pub token_b_vault: [u8; 32],
    // 288-319: Whitelisted vault address
    pub whitelisted_vault: [u8; 32],
    // 320-351: Partner address
    pub partner: [u8; 32],
    // 352-367: Liquidity (u128, 16 bytes)
    pub liquidity: [u8; 16],
    // 368-383: Padding/reserve (u128, 16 bytes)
    pub _padding: [u8; 16],
    // 384-391: Protocol A fee (u64, 8 bytes)
    pub protocol_a_fee: [u8; 8],
    // 392-399: Protocol B fee (u64, 8 bytes)
    pub protocol_b_fee: [u8; 8],
    // 400-407: Partner A fee (u64, 8 bytes)
    pub partner_a_fee: [u8; 8],
    // 408-415: Partner B fee (u64, 8 bytes)
    pub partner_b_fee: [u8; 8],
    // 416-431: Min price (sqrt, u128, 16 bytes)
    pub sqrt_min_price: [u8; 16],
    // 432-447: Max price (sqrt, u128, 16 bytes)
    pub sqrt_max_price: [u8; 16],
    // 448-463: Current price (sqrt, u128, 16 bytes)
    pub sqrt_price: [u8; 16],
    // 464-471: Activation point (u64, 8 bytes)
    pub activation_point: [u8; 8],
    // 472: Activation type (u8, 1 byte)
    pub activation_type: u8,
    // 473: Pool status (u8, 1 byte)
    pub pool_status: u8,
    // 474: Token A flag (u8, 1 byte)
    pub token_a_flag: u8,
    // 475: Token B flag (u8, 1 byte)
    pub token_b_flag: u8,
    // 476: Collect fee mode (u8, 1 byte)
    pub collect_fee_mode: u8,
    // 477: Pool type (u8, 1 byte)
    pub pool_type: u8,
    // 478: Pool version (u8, 1 byte)
    pub version: u8,
    // 479: Padding (u8, 1 byte)
    pub _padding_0: u8,
    // 480-511: Fee A per liquidity (U256, 32 bytes)
    pub fee_a_per_liquidity: [u8; 32],
    // 512-543: Fee B per liquidity (U256, 32 bytes)
    pub fee_b_per_liquidity: [u8; 32],
    // 544-559: Permanent lock liquidity (u128, 16 bytes)
    pub permanent_lock_liquidity: [u8; 16],
    // 560-639: Pool metrics (80 bytes)
    pub metrics: [u8; 80],
    // 640-671: Pool creator (32 bytes)
    pub creator: [u8; 32],
    // 672-719: Padding (6 * u64 = 48 bytes)
    pub _padding_1: [u8; 48],
    // 720-1103: Reward infos (3 * 128 = 384 bytes)
    pub reward_infos: [[u8; 128]; 3],
}

/// DAMMV2 pool parsed data structure
#[derive(Debug, Clone)]
pub struct DAMMV2LiquidityPoolData {
    // Pool fees structure - simplified for now
    pub pool_fees: PoolFeesData,
    // Token addresses
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub whitelisted_vault: Pubkey,
    pub partner: Pubkey,
    pub creator: Pubkey,
    // Liquidity and fees
    pub liquidity: u128,
    pub protocol_a_fee: u64,
    pub protocol_b_fee: u64,
    pub partner_a_fee: u64,
    pub partner_b_fee: u64,
    pub permanent_lock_liquidity: u128,
    // Price information (sqrt prices)
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub sqrt_price: u128,
    // Activation info
    pub activation_point: u64,
    pub activation_type: u8,
    // Status flags
    pub pool_status: u8,
    pub token_a_flag: u8,
    pub token_b_flag: u8,
    pub collect_fee_mode: u8,
    pub pool_type: u8,
    pub version: u8,
    // Cumulative fees per liquidity
    pub fee_a_per_liquidity: [u8; 32], // U256
    pub fee_b_per_liquidity: [u8; 32], // U256
    // Pool metrics
    pub metrics: PoolMetricsData,
    // Reward information
    pub reward_infos: [RewardInfoData; 3],
}

/// Pool fees data structure
#[derive(Debug, Clone, Default)]
pub struct PoolFeesData {
    // Simplified - actual structure is more complex
    pub base_fee_numerator: u32,
    pub base_fee_denominator: u32,
    pub dynamic_fee: DynamicFeeData,
}

/// Dynamic fee data for DAMMV2
#[derive(Debug, Clone, Default)]
pub struct DynamicFeeData {
    pub is_dynamic_fee_enabled: bool,
    pub bin_step: u128,
    pub volatility_reference: u128,
    pub volatility_accumulator: u128,
    pub last_update_timestamp: u64,
    pub activation_point: u64,
}

/// Pool metrics data
#[derive(Debug, Clone)]
pub struct PoolMetricsData {
    pub total_lp_a_fee: u128,
    pub total_lp_b_fee: u128,
    pub total_protocol_a_fee: u64,
    pub total_protocol_b_fee: u64,
    pub total_partner_a_fee: u64,
    pub total_partner_b_fee: u64,
    pub total_position: u64,
}

/// Reward info data
#[derive(Debug, Clone)]
pub struct RewardInfoData {
    pub initialized: u8,
    pub reward_token_flag: u8,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub reward_duration: u64,
    pub reward_duration_end: u64,
    pub reward_rate: u128,
    pub reward_per_token_stored: [u8; 32], // U256
    pub last_update_time: u64,
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}

impl DAMMV2LiquidityPool {
    /// Parse DAMMV2 pool data from byte array
    pub fn get_liquidity_pool_info(data: &[u8]) -> Result<DAMMV2LiquidityPoolData, String> {
        let total_expected_size = DISCRIMINATOR_LEN + DAMMV2_LIQUIDITY_POOL_DATA_SIZE;
        if data.len() != total_expected_size {
            return Err(format!(
                "DAMMV2 pool data size mismatch. Expected {} (1104 data + 8 discriminator), got {}",
                total_expected_size,
                data.len()
            ));
        }
        let mut offset: usize = DISCRIMINATOR_LEN;
        let read_pubkey = |d: &[u8], o: &mut usize| -> Result<Pubkey, String> {
            if *o + 32 > d.len() {
                return Err(format!(
                    "Reading pubkey out of bounds: offset={}, len={}",
                    *o,
                    d.len()
                ));
            }
            let pk = Pubkey::new_from_array(d[*o..*o + 32].try_into().unwrap());
            *o += 32;
            Ok(pk)
        };
        let read_u8 = |d: &[u8], o: &mut usize| -> Result<u8, String> {
            if *o >= d.len() {
                return Err(format!(
                    "Reading u8 out of bounds: offset={}, len={}",
                    *o,
                    d.len()
                ));
            }
            let v = d[*o];
            *o += 1;
            Ok(v)
        };
        let read_u64 = |d: &[u8], o: &mut usize| -> Result<u64, String> {
            if *o + 8 > d.len() {
                return Err(format!(
                    "Reading u64 out of bounds: offset={}, len={}",
                    *o,
                    d.len()
                ));
            }
            let v = u64::from_le_bytes(d[*o..*o + 8].try_into().unwrap());
            *o += 8;
            Ok(v)
        };
        let read_u128 = |d: &[u8], o: &mut usize| -> Result<u128, String> {
            if *o + 16 > d.len() {
                return Err(format!(
                    "Reading u128 out of bounds: offset={}, len={}",
                    *o,
                    d.len()
                ));
            }
            let v = u128::from_le_bytes(d[*o..*o + 16].try_into().unwrap());
            *o += 16;
            Ok(v)
        };
        let read_bytes = |d: &[u8], o: &mut usize, len: usize| -> Result<Vec<u8>, String> {
            if *o + len > d.len() {
                return Err(format!(
                    "Reading {} bytes out of bounds: offset={}, len={}",
                    len,
                    *o,
                    d.len()
                ));
            }
            let bytes = d[*o..*o + len].to_vec();
            *o += len;
            Ok(bytes)
        };
        let pool_fees_bytes = read_bytes(data, &mut offset, 160)?;
        let pool_fees = parse_pool_fees(&pool_fees_bytes);
        let token_a_mint = read_pubkey(data, &mut offset)?;
        let token_b_mint = read_pubkey(data, &mut offset)?;
        let token_a_vault = read_pubkey(data, &mut offset)?;
        let token_b_vault = read_pubkey(data, &mut offset)?;
        let whitelisted_vault = read_pubkey(data, &mut offset)?;
        let partner = read_pubkey(data, &mut offset)?;
        let liquidity = read_u128(data, &mut offset)?;
        let _padding_reserve = read_u128(data, &mut offset)?;
        let protocol_a_fee = read_u64(data, &mut offset)?;
        let protocol_b_fee = read_u64(data, &mut offset)?;
        let partner_a_fee = read_u64(data, &mut offset)?;
        let partner_b_fee = read_u64(data, &mut offset)?;
        let sqrt_min_price = read_u128(data, &mut offset)?;
        let sqrt_max_price = read_u128(data, &mut offset)?;
        let sqrt_price = read_u128(data, &mut offset)?;
        let activation_point = read_u64(data, &mut offset)?;
        let activation_type = read_u8(data, &mut offset)?;
        let pool_status = read_u8(data, &mut offset)?;
        let token_a_flag = read_u8(data, &mut offset)?;
        let token_b_flag = read_u8(data, &mut offset)?;
        let collect_fee_mode = read_u8(data, &mut offset)?;
        let pool_type = read_u8(data, &mut offset)?;
        let version = read_u8(data, &mut offset)?;
        let _padding_0 = read_u8(data, &mut offset)?;
        let fee_a_per_liquidity_bytes = read_bytes(data, &mut offset, 32)?;
        let fee_b_per_liquidity_bytes = read_bytes(data, &mut offset, 32)?;
        let fee_a_per_liquidity: [u8; 32] = fee_a_per_liquidity_bytes
            .try_into()
            .map_err(|_| "Failed to convert fee_a_per_liquidity bytes".to_string())?;
        let fee_b_per_liquidity: [u8; 32] = fee_b_per_liquidity_bytes
            .try_into()
            .map_err(|_| "Failed to convert fee_b_per_liquidity bytes".to_string())?;
        let permanent_lock_liquidity = read_u128(data, &mut offset)?;
        let metrics_bytes = read_bytes(data, &mut offset, 80)?;
        let metrics = parse_pool_metrics(&metrics_bytes);
        let creator = read_pubkey(data, &mut offset)?;
        offset += 48;
        let mut reward_infos: [RewardInfoData; 3] = std::array::from_fn(|_| RewardInfoData {
            initialized: 0,
            reward_token_flag: 0,
            mint: Pubkey::default(),
            vault: Pubkey::default(),
            funder: Pubkey::default(),
            reward_duration: 0,
            reward_duration_end: 0,
            reward_rate: 0,
            reward_per_token_stored: [0; 32],
            last_update_time: 0,
            cumulative_seconds_with_empty_liquidity_reward: 0,
        });
        for i in 0..3 {
            let start_offset = offset;
            if offset >= data.len() {
                break;
            }
            reward_infos[i].initialized = read_u8(data, &mut offset)?;
            if reward_infos[i].initialized == 0 {
                if i == 0 {
                    offset = 600 + 184;
                } else if i == 1 {
                    offset = 600 + 184 * 2;
                } else if i == 2 {
                    offset = data.len();
                }
                continue;
            }
            reward_infos[i].reward_token_flag = read_u8(data, &mut offset)?;
            offset += 6;
            reward_infos[i].mint = read_pubkey(data, &mut offset)?;
            reward_infos[i].vault = read_pubkey(data, &mut offset)?;
            reward_infos[i].funder = read_pubkey(data, &mut offset)?;
            reward_infos[i].reward_duration = read_u64(data, &mut offset)?;
            reward_infos[i].reward_duration_end = read_u64(data, &mut offset)?;
            reward_infos[i].reward_rate = read_u128(data, &mut offset)?;
            let reward_per_token_bytes = read_bytes(data, &mut offset, 32)?;
            reward_infos[i].reward_per_token_stored = reward_per_token_bytes
                .try_into()
                .map_err(|_| "Failed to convert reward_per_token_stored bytes".to_string())?;
            reward_infos[i].last_update_time = read_u64(data, &mut offset)?;
            reward_infos[i].cumulative_seconds_with_empty_liquidity_reward =
                read_u64(data, &mut offset)?;
        }
        if offset != data.len() {
            let remaining = data.len() - offset;
            if remaining > 0 {
                offset = data.len();
            }
        }
        Ok(DAMMV2LiquidityPoolData {
            pool_fees,
            token_a_mint,
            token_b_mint,
            token_a_vault,
            token_b_vault,
            whitelisted_vault,
            partner,
            creator,
            liquidity,
            protocol_a_fee,
            protocol_b_fee,
            partner_a_fee,
            partner_b_fee,
            permanent_lock_liquidity,
            sqrt_min_price,
            sqrt_max_price,
            sqrt_price,
            activation_point,
            activation_type,
            pool_status,
            token_a_flag,
            token_b_flag,
            collect_fee_mode,
            pool_type,
            version,
            fee_a_per_liquidity,
            fee_b_per_liquidity,
            metrics,
            reward_infos,
        })
    }

    // Getter methods for public keys
    pub fn token_a_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_a_mint)
    }

    pub fn token_b_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_b_mint)
    }

    pub fn token_a_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_a_vault)
    }

    pub fn token_b_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_b_vault)
    }

    pub fn whitelisted_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.whitelisted_vault)
    }

    pub fn partner_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.partner)
    }

    pub fn creator_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.creator)
    }
}

impl DAMMV2LiquidityPoolData {
    /// Get current price from sqrt price
    pub fn get_price(&self) -> f64 {
        let sqrt_price_f64 = self.sqrt_price as f64;
        let sqrt_price = sqrt_price_f64 / (1u128 << 64) as f64;
        let price = sqrt_price * sqrt_price;
        price
    }
    /// Get tick-based price (for compatibility with CLMM)
    pub fn get_tick_price(&self) -> f64 {
        let price = self.get_price();
        price
    }
    /// Get token A amount in the pool (approximation)
    pub fn get_token_a_amount(&self) -> f64 {
        if self.liquidity == 0 {
            return 0.0;
        }
        let sqrt_price = (self.sqrt_price as f64) / (1u128 << 64) as f64;
        let liquidity = self.liquidity as f64;
        liquidity / sqrt_price
    }

    /// Get token B amount in the pool (approximation)
    pub fn get_token_b_amount(&self) -> f64 {
        if self.liquidity == 0 {
            return 0.0;
        }
        // DAMMV2 uses dynamic weights, so this is an approximation
        let sqrt_price = (self.sqrt_price as f64) / (1u128 << 64) as f64;
        let liquidity = self.liquidity as f64;
        // Simplified calculation
        liquidity * sqrt_price
    }
    /// Check if dynamic fee is enabled
    pub fn is_dynamic_fee_enabled(&self) -> bool {
        self.pool_fees.dynamic_fee.is_dynamic_fee_enabled
    }
    /// Get pool status as string
    pub fn get_pool_status_str(&self) -> &'static str {
        match self.pool_status {
            0 => "Enable",
            1 => "Disable",
            _ => "Unknown",
        }
    }
    /// Get collect fee mode as string
    pub fn get_collect_fee_mode_str(&self) -> &'static str {
        match self.collect_fee_mode {
            0 => "BothToken",
            1 => "OnlyA",
            2 => "OnlyB",
            _ => "Unknown",
        }
    }
    /// Get pool type as string
    pub fn get_pool_type_str(&self) -> &'static str {
        match self.pool_type {
            0 => "Permissionless",
            1 => "Customizable",
            _ => "Unknown",
        }
    }
    /// Get activation type as string
    pub fn get_activation_type_str(&self) -> &'static str {
        match self.activation_type {
            0 => "BySlot",
            1 => "ByTimestamp",
            _ => "Unknown",
        }
    }
}

fn parse_pool_fees(bytes: &[u8]) -> PoolFeesData {
    if bytes.len() < 160 {
        println!(
            "Warning: PoolFeesStruct data too short: {} < 160",
            bytes.len()
        );
        return PoolFeesData::default();
    }
    let mut offset = 0;
    let cliff_fee_numerator =
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    let base_fee_mode = bytes[offset];
    offset += 1;
    offset += 5;
    let first_factor = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap_or([0; 2]));
    offset += 2;
    let second_factor = bytes[offset..offset + 8].try_into().unwrap_or([0; 8]);
    offset += 8;
    let third_factor = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    offset += 8;
    let protocol_fee_percent = bytes[offset];
    offset += 1;
    let partner_fee_percent = bytes[offset];
    offset += 1;
    let referral_fee_percent = bytes[offset];
    offset += 1;
    offset += 5;
    let dynamic_initialized = bytes[offset];
    offset += 1;
    offset += 7;
    let max_volatility_accumulator =
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
    offset += 4;
    let variable_fee_control =
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
    offset += 4;
    let bin_step = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap_or([0; 2]));
    offset += 2;
    let filter_period = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap_or([0; 2]));
    offset += 2;
    let decay_period = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap_or([0; 2]));
    offset += 2;
    let reduction_factor =
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap_or([0; 2]));
    offset += 2;
    let last_update_timestamp =
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    let bin_step_u128 =
        u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap_or([0; 16]));
    offset += 16;
    let sqrt_price_reference =
        u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap_or([0; 16]));
    offset += 16;
    let volatility_accumulator =
        u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap_or([0; 16]));
    offset += 16;
    let volatility_reference =
        u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap_or([0; 16]));
    offset += 16;
    offset += 16;
    let base_fee_numerator = cliff_fee_numerator;
    let base_fee_denominator = 10000; // FEE_DENOMINATOR
    let is_dynamic_fee_enabled = dynamic_initialized != 0;
    PoolFeesData {
        base_fee_numerator: base_fee_numerator as u32,
        base_fee_denominator: base_fee_denominator as u32,
        dynamic_fee: DynamicFeeData {
            is_dynamic_fee_enabled,
            bin_step: bin_step_u128,
            volatility_reference,
            volatility_accumulator,
            last_update_timestamp,
            activation_point: 0,
        },
    }
}

/// Parse pool metrics from bytes
fn parse_pool_metrics(bytes: &[u8]) -> PoolMetricsData {
    if bytes.len() < 80 {
        return PoolMetricsData {
            total_lp_a_fee: 0,
            total_lp_b_fee: 0,
            total_protocol_a_fee: 0,
            total_protocol_b_fee: 0,
            total_partner_a_fee: 0,
            total_partner_b_fee: 0,
            total_position: 0,
        };
    }
    let mut offset = 0;
    let total_lp_a_fee =
        u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap_or([0; 16]));
    offset += 16;
    let total_lp_b_fee =
        u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap_or([0; 16]));
    offset += 16;
    let total_protocol_a_fee =
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    let total_protocol_b_fee =
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    let total_partner_a_fee =
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    let total_partner_b_fee =
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    offset += 8;
    let total_position = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
    PoolMetricsData {
        total_lp_a_fee,
        total_lp_b_fee,
        total_protocol_a_fee,
        total_protocol_b_fee,
        total_partner_a_fee,
        total_partner_b_fee,
        total_position,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::Meteora;
    use solana_network_sdk::Solana;

    #[tokio::test]
    async fn test_dammv2_pool_parsing() {
        let sol = Solana::new(solana_network_sdk::types::Mode::MAIN).unwrap();
        let meteora = Meteora::new(Arc::new(sol));
        let pool_data = meteora
            .get_liquidity_pool_dynv2("5gB4NPgFB3MHFHSeKN4sbaY6t9MB8ikCe9HyiKYid4Td")
            .await
            .unwrap();
        println!("Pool Data: {:?}", pool_data);
    }
}

use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

pub const DLMM_LIQUIDITY_POOL_DATA_SIZE: usize = 896; // 8 + 896 = 904
const DISCRIMINATOR_LEN: usize = 8;

unsafe impl Pod for DLMMLiquidityPool {}
unsafe impl Zeroable for DLMMLiquidityPool {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DLMMLiquidityPool {
    pub parameters: [u8; 32],                  // StaticParameters 32 bytes
    pub v_parameters: [u8; 32],                // VariableParameters 32 bytes
    pub bump_seed: [u8; 1],                    // u8[1]
    pub bin_step_seed: [u8; 2],                // u8[2]
    pub pair_type: u8,                         // u8
    pub active_id: [u8; 4],                    // i32
    pub bin_step: [u8; 2],                     // u16
    pub status: u8,                            // u8
    pub require_base_factor_seed: u8,          // u8
    pub base_factor_seed: [u8; 2],             // u8[2]
    pub activation_type: u8,                   // u8
    pub creator_pool_on_off_control: u8,       // u8
    pub token_x_mint: [u8; 32],                // Pubkey
    pub token_y_mint: [u8; 32],                // Pubkey
    pub reserve_x: [u8; 32],                   // Pubkey
    pub reserve_y: [u8; 32],                   // Pubkey
    pub protocol_fee: [u8; 16],                // ProtocolFee 16 bytes
    pub padding1: [u8; 32],                    // padding1 (32 bytes)
    pub reward_infos: [[u8; 144]; 2],          // RewardInfo 144 bytes * 2 = 288 bytes
    pub oracle: [u8; 32],                      // Pubkey
    pub bin_array_bitmap: [[u8; 8]; 16],       // [u64; 16] = 128 bytes
    pub last_updated_at: [u8; 8],              // i64
    pub padding2: [u8; 32],                    // padding2 (32 bytes)
    pub pre_activation_swap_address: [u8; 32], // Pubkey
    pub base_key: [u8; 32],                    // Pubkey
    pub activation_point: [u8; 8],             // u64
    pub pre_activation_duration: [u8; 8],      // u64
    pub padding3: [u8; 8],                     // padding3 (8 bytes)
    pub padding4: [u8; 8],                     // u64 (padding4)
    pub creator: [u8; 32],                     // Pubkey
    pub token_mint_x_program_flag: u8,         // u8
    pub token_mint_y_program_flag: u8,         // u8
    pub reserved: [u8; 22],                    // reserved (22 bytes)
}

/// DLMM pool parsed data structure
#[derive(Debug, Clone)]
pub struct DLMMLiquidityPoolData {
    // Token addresses
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub oracle: Pubkey,
    pub creator: Pubkey,
    pub base_key: Pubkey,
    pub pre_activation_swap_address: Pubkey,
    // Pair configuration
    pub bin_step: u16,
    pub active_id: i32,
    pub pair_type: u8,
    pub status: u8,
    pub activation_type: u8,
    pub bump_seed: u8,
    pub require_base_factor_seed: u8,
    pub static_parameters: StaticParameters,
    pub variable_parameters: VariableParameters,
    pub protocol_fee: ProtocolFee,
    pub reward_infos: [RewardInfo; 2],
    pub bin_array_bitmap: [u64; 16],
    pub activation_point: u64,
    pub pre_activation_duration: u64,
    pub last_updated_at: i64,
    pub token_mint_x_program_flag: u8,
    pub token_mint_y_program_flag: u8,
    pub bin_step_seed: u16,
    pub base_factor_seed: u16,
}

/// Static parameters structure
#[derive(Debug, Clone)]
pub struct StaticParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub min_bin_id: i32,
    pub max_bin_id: i32,
    pub protocol_share: u16,
}

/// Variable parameters structure
#[derive(Debug, Clone)]
pub struct VariableParameters {
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub index_reference: i32,
    pub last_update_timestamp: i64,
}

/// Protocol fee structure
#[derive(Debug, Clone)]
pub struct ProtocolFee {
    pub amount_x: u64,
    pub amount_y: u64,
}

/// Reward information structure
#[derive(Debug, Clone)]
pub struct RewardInfo {
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub funder: Pubkey,
    pub reward_duration: u64,
    pub reward_duration_end: u64,
    pub reward_rate: u128,
    pub last_update_time: i64,
    pub cumulative_seconds_with_empty_liquidity_reward: u64,
}

impl DLMMLiquidityPool {
    /// Parse DLMM pool data from byte array
    pub fn get_liquidity_pool_info(data: &[u8]) -> Result<DLMMLiquidityPoolData, String> {
        let total_expected_size = DISCRIMINATOR_LEN + DLMM_LIQUIDITY_POOL_DATA_SIZE;
        if data.len() != total_expected_size {
            return Err(format!(
                "DLMM pool data size mismatch. Expected {} ({} data + {} discriminator), got {}",
                total_expected_size,
                DLMM_LIQUIDITY_POOL_DATA_SIZE,
                DISCRIMINATOR_LEN,
                data.len()
            ));
        }
        let pool_data = &data[DISCRIMINATOR_LEN..];
        let struct_size = std::mem::size_of::<DLMMLiquidityPool>();
        if struct_size != DLMM_LIQUIDITY_POOL_DATA_SIZE {
            return Err(format!(
                "Struct size mismatch. DLMMLiquidityPool is {} bytes, but DLMM_LIQUIDITY_POOL_DATA_SIZE is {}",
                struct_size, DLMM_LIQUIDITY_POOL_DATA_SIZE
            ));
        }
        let pool: &DLMMLiquidityPool = bytemuck::try_from_bytes(pool_data)
            .map_err(|e| format!("Failed to cast bytes to DLMMLiquidityPool: {}", e))?;
        Self::parse_from_struct(pool)
    }

    fn parse_from_struct(pool: &DLMMLiquidityPool) -> Result<DLMMLiquidityPoolData, String> {
        let static_parameters = StaticParameters {
            base_factor: u16::from_le_bytes([pool.parameters[0], pool.parameters[1]]),
            filter_period: u16::from_le_bytes([pool.parameters[2], pool.parameters[3]]),
            decay_period: u16::from_le_bytes([pool.parameters[4], pool.parameters[5]]),
            reduction_factor: u16::from_le_bytes([pool.parameters[6], pool.parameters[7]]),
            variable_fee_control: u32::from_le_bytes([
                pool.parameters[8],
                pool.parameters[9],
                pool.parameters[10],
                pool.parameters[11],
            ]),
            max_volatility_accumulator: u32::from_le_bytes([
                pool.parameters[12],
                pool.parameters[13],
                pool.parameters[14],
                pool.parameters[15],
            ]),
            min_bin_id: i32::from_le_bytes([
                pool.parameters[16],
                pool.parameters[17],
                pool.parameters[18],
                pool.parameters[19],
            ]),
            max_bin_id: i32::from_le_bytes([
                pool.parameters[20],
                pool.parameters[21],
                pool.parameters[22],
                pool.parameters[23],
            ]),
            protocol_share: u16::from_le_bytes([pool.parameters[24], pool.parameters[25]]),
        };
        let variable_parameters = VariableParameters {
            volatility_accumulator: u32::from_le_bytes([
                pool.v_parameters[0],
                pool.v_parameters[1],
                pool.v_parameters[2],
                pool.v_parameters[3],
            ]),
            volatility_reference: u32::from_le_bytes([
                pool.v_parameters[4],
                pool.v_parameters[5],
                pool.v_parameters[6],
                pool.v_parameters[7],
            ]),
            index_reference: i32::from_le_bytes([
                pool.v_parameters[8],
                pool.v_parameters[9],
                pool.v_parameters[10],
                pool.v_parameters[11],
            ]),
            last_update_timestamp: i64::from_le_bytes([
                pool.v_parameters[16],
                pool.v_parameters[17],
                pool.v_parameters[18],
                pool.v_parameters[19],
                pool.v_parameters[20],
                pool.v_parameters[21],
                pool.v_parameters[22],
                pool.v_parameters[23],
            ]),
        };
        let protocol_fee = ProtocolFee {
            amount_x: u64::from_le_bytes([
                pool.protocol_fee[0],
                pool.protocol_fee[1],
                pool.protocol_fee[2],
                pool.protocol_fee[3],
                pool.protocol_fee[4],
                pool.protocol_fee[5],
                pool.protocol_fee[6],
                pool.protocol_fee[7],
            ]),
            amount_y: u64::from_le_bytes([
                pool.protocol_fee[8],
                pool.protocol_fee[9],
                pool.protocol_fee[10],
                pool.protocol_fee[11],
                pool.protocol_fee[12],
                pool.protocol_fee[13],
                pool.protocol_fee[14],
                pool.protocol_fee[15],
            ]),
        };
        let mut reward_infos = Vec::new();
        for i in 0..2 {
            let reward_data = &pool.reward_infos[i];
            reward_infos.push(RewardInfo {
                mint: Pubkey::new_from_array(reward_data[0..32].try_into().unwrap()),
                vault: Pubkey::new_from_array(reward_data[32..64].try_into().unwrap()),
                funder: Pubkey::new_from_array(reward_data[64..96].try_into().unwrap()),
                reward_duration: u64::from_le_bytes(reward_data[96..104].try_into().unwrap()),
                reward_duration_end: u64::from_le_bytes(reward_data[104..112].try_into().unwrap()),
                reward_rate: u128::from_le_bytes(reward_data[112..128].try_into().unwrap()),
                last_update_time: i64::from_le_bytes(reward_data[128..136].try_into().unwrap()),
                cumulative_seconds_with_empty_liquidity_reward: u64::from_le_bytes(
                    reward_data[136..144].try_into().unwrap(),
                ),
            });
        }
        let mut bin_array_bitmap = [0u64; 16];
        for i in 0..16 {
            bin_array_bitmap[i] = u64::from_le_bytes(pool.bin_array_bitmap[i]);
        }
        Ok(DLMMLiquidityPoolData {
            token_x_mint: Pubkey::new_from_array(pool.token_x_mint),
            token_y_mint: Pubkey::new_from_array(pool.token_y_mint),
            reserve_x: Pubkey::new_from_array(pool.reserve_x),
            reserve_y: Pubkey::new_from_array(pool.reserve_y),
            oracle: Pubkey::new_from_array(pool.oracle),
            creator: Pubkey::new_from_array(pool.creator),
            base_key: Pubkey::new_from_array(pool.base_key),
            pre_activation_swap_address: Pubkey::new_from_array(pool.pre_activation_swap_address),
            bin_step: u16::from_le_bytes(pool.bin_step),
            active_id: i32::from_le_bytes(pool.active_id),
            pair_type: pool.pair_type,
            status: pool.status,
            activation_type: pool.activation_type,
            bump_seed: pool.bump_seed[0],
            require_base_factor_seed: pool.require_base_factor_seed,
            static_parameters,
            variable_parameters,
            protocol_fee,
            reward_infos: reward_infos
                .try_into()
                .map_err(|_| "Failed to convert reward_infos to array".to_string())?,
            bin_array_bitmap,
            activation_point: u64::from_le_bytes(pool.activation_point),
            pre_activation_duration: u64::from_le_bytes(pool.pre_activation_duration),
            last_updated_at: i64::from_le_bytes(pool.last_updated_at),
            token_mint_x_program_flag: pool.token_mint_x_program_flag,
            token_mint_y_program_flag: pool.token_mint_y_program_flag,
            bin_step_seed: u16::from_le_bytes(pool.bin_step_seed),
            base_factor_seed: u16::from_le_bytes(pool.base_factor_seed),
        })
    }

    // Getter methods for public keys
    pub fn token_x_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_x_mint)
    }

    pub fn token_y_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_y_mint)
    }

    pub fn reserve_x_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.reserve_x)
    }

    pub fn reserve_y_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.reserve_y)
    }

    pub fn oracle_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.oracle)
    }

    pub fn creator_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.creator)
    }

    pub fn base_key_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.base_key)
    }

    pub fn pre_activation_swap_address_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.pre_activation_swap_address)
    }
}

impl DLMMLiquidityPoolData {
    /// Get pool status as string
    pub fn get_status_str(&self) -> &'static str {
        match self.status {
            0 => "Enabled",
            1 => "Disabled",
            2 => "Bootstrap",
            _ => "Unknown",
        }
    }

    /// Get activation type as string
    pub fn get_activation_type_str(&self) -> &'static str {
        match self.activation_type {
            0 => "Slot",
            1 => "Timestamp",
            _ => "Unknown",
        }
    }

    /// Get pair type as string
    pub fn get_pair_type_str(&self) -> &'static str {
        match self.pair_type {
            0 => "Permissionless",
            1 => "Permissioned",
            2 => "Stable",
            _ => "Unknown",
        }
    }

    /// Get token amounts from reserves
    pub fn get_token_x_amount(&self) -> f64 {
        // get account balance
        0.0
    }

    pub fn get_token_y_amount(&self) -> f64 {
        // get account balance
        0.0
    }

    /// Get current price from active_id
    pub fn get_price(&self) -> f64 {
        let price = (1.0001f64).powi(self.active_id);
        price
    }

    /// Get fee rate
    pub fn get_fee_rate(&self) -> f64 {
        let base_fee = self.static_parameters.base_factor as f64;
        let variable_fee = self.variable_parameters.volatility_accumulator as f64;
        (base_fee + variable_fee) / 10000.0
    }

    /// Check if pool is enabled
    pub fn is_enabled(&self) -> bool {
        self.status == 0
    }

    /// Check if pool is in bootstrap mode
    pub fn is_bootstrap(&self) -> bool {
        self.status == 2
    }

    /// Get protocol fees
    pub fn get_protocol_fees(&self) -> (u64, u64) {
        (self.protocol_fee.amount_x, self.protocol_fee.amount_y)
    }

    /// Get pre-activation swap address
    pub fn get_pre_activation_swap_address(&self) -> Pubkey {
        self.pre_activation_swap_address
    }
}

#[cfg(test)]
mod tests {
    use crate::Meteora;
    use solana_network_client::{Mode, SolanaClient};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_dlmm_pool_parsing() {
        let solana_client = SolanaClient::new(Mode::MAIN).unwrap();
        let meteora = Meteora::new(Arc::new(solana_client));
        let pool_data = meteora
            .get_liquidity_pool_dlmm("BjxkogRUDnb72MSBTfsyuq54yntqxyVKozK9WywMszvZ")
            .await
            .unwrap();
        println!("DLMM Pool Data: {:?}", pool_data);
    }
}

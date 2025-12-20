use bytemuck::{Pod, Zeroable};
use solana_sdk::pubkey::Pubkey;

/// V1 liquidity pool data size
pub const DAMM_LIQUIDITY_POOL_DATA_SIZE: usize = 944; // 952 total - 8 discriminator
const DISCRIMINATOR_LEN: usize = 8;

unsafe impl Pod for DAMMLiquidityPool {}
unsafe impl Zeroable for DAMMLiquidityPool {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DAMMLiquidityPool {
    pub lp_mint: [u8; 32],
    pub token_a_mint: [u8; 32],
    pub token_b_mint: [u8; 32],
    pub a_vault: [u8; 32],
    pub b_vault: [u8; 32],
    pub a_vault_lp: [u8; 32],
    pub b_vault_lp: [u8; 32],
    pub a_vault_lp_bump: u8,
    pub enabled: u8,
    pub protocol_token_a_fee: [u8; 32],
    pub protocol_token_b_fee: [u8; 32],
    pub fee_last_updated_at: [u8; 8],
    pub _padding0: [u8; 24],
    pub fees: [u8; 48],
    pub pool_type: u8,
    pub stake: [u8; 32],
    pub total_locked_lp: [u8; 8],
    pub bootstrapping: [u8; 48],
    pub partner_info: [u8; 56],
    pub padding: [u8; 429],
    pub curve_type: [u8; 8],
}

/// V1 pool parsed data structure
#[derive(Debug, Clone)]
pub struct DAMMLiquidityPoolData {
    // Token addresses
    pub lp_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub a_vault: Pubkey,
    pub b_vault: Pubkey,
    pub a_vault_lp: Pubkey,
    pub b_vault_lp: Pubkey,
    pub protocol_token_a_fee: Pubkey,
    pub protocol_token_b_fee: Pubkey,
    pub stake: Pubkey,
    // Pool status and configuration
    pub a_vault_lp_bump: u8,
    pub enabled: bool,
    pub fee_last_updated_at: u64,
    pub pool_type: PoolType,
    pub total_locked_lp: u64,
    // Fee structure
    pub fees: DynPoolFees,
    // Bootstrapping information
    pub bootstrapping: BootstrappingInfo,
    // Partner information
    pub partner_info: PartnerInfo,
    // Curve type
    pub curve_type: CurveTypeInfo,
}

/// Pool type enum
#[derive(Debug, Clone, PartialEq)]
pub enum PoolType {
    Permissioned,
    Permissionless,
    Unknown(u8),
}

/// Pool fees data structure
#[derive(Debug, Clone)]
pub struct DynPoolFees {
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub protocol_trade_fee_numerator: u64,
    pub protocol_trade_fee_denominator: u64,
}

/// Bootstrapping information
#[derive(Debug, Clone)]
pub struct BootstrappingInfo {
    pub activation_point: u64,
    pub whitelisted_vault: Pubkey,
    pub pool_creator: Pubkey,
    pub activation_type: ActivationType,
}

/// Activation type enum
#[derive(Debug, Clone, PartialEq)]
pub enum ActivationType {
    Slot,
    Timestamp,
    Unknown(u8),
}

/// Partner information
#[derive(Debug, Clone)]
pub struct PartnerInfo {
    pub fee_numerator: u64,
    pub partner_authority: Pubkey,
    pub pending_fee_a: u64,
    pub pending_fee_b: u64,
}

/// Curve type information
#[derive(Debug, Clone)]
pub enum CurveTypeInfo {
    ConstantProduct,
    Stable {
        amp: u64,
        token_multiplier: TokenMultiplier,
        depeg: DepegInfo,
        last_amp_updated_timestamp: u64,
    },
    Unknown,
}

/// Token multiplier
#[derive(Debug, Clone)]
pub struct TokenMultiplier {
    pub token_a_multiplier: u64,
    pub token_b_multiplier: u64,
    pub precision_factor: u8,
}

/// Depeg information
#[derive(Debug, Clone)]
pub struct DepegInfo {
    pub base_virtual_price: u64,
    pub base_cache_updated: u64,
    pub depeg_type: DepegType,
}

/// Depeg type
#[derive(Debug, Clone, PartialEq)]
pub enum DepegType {
    None,
    Marinade,
    Lido,
    SplStake,
    Unknown,
}

impl DAMMLiquidityPool {
    /// Parse V1 pool data from byte array
    pub fn get_liquidity_pool_info(data: &[u8]) -> Result<DAMMLiquidityPoolData, String> {
        let total_expected_size = DISCRIMINATOR_LEN + DAMM_LIQUIDITY_POOL_DATA_SIZE;
        if data.len() != total_expected_size {
            return Err(format!(
                "V1 pool data size mismatch. Expected {} (944 data + 8 discriminator), got {}",
                total_expected_size,
                data.len()
            ));
        }
        let mut offset: usize = DISCRIMINATOR_LEN;
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
        let lp_mint = read_pubkey(data, &mut offset)?;
        let token_a_mint = read_pubkey(data, &mut offset)?;
        let token_b_mint = read_pubkey(data, &mut offset)?;
        let a_vault = read_pubkey(data, &mut offset)?;
        let b_vault = read_pubkey(data, &mut offset)?;
        let a_vault_lp = read_pubkey(data, &mut offset)?;
        let b_vault_lp = read_pubkey(data, &mut offset)?;
        let a_vault_lp_bump = read_u8(data, &mut offset)?;
        let enabled_byte = read_u8(data, &mut offset)?;
        let enabled = enabled_byte != 0;
        let protocol_token_a_fee = read_pubkey(data, &mut offset)?;
        let protocol_token_b_fee = read_pubkey(data, &mut offset)?;
        let fee_last_updated_at = read_u64(data, &mut offset)?;
        // Skip padding0
        offset += 24;
        // Parse fees
        let trade_fee_numerator = read_u64(data, &mut offset)?;
        let trade_fee_denominator = read_u64(data, &mut offset)?;
        let protocol_trade_fee_numerator = read_u64(data, &mut offset)?;
        let protocol_trade_fee_denominator = read_u64(data, &mut offset)?;
        let fees = DynPoolFees {
            trade_fee_numerator,
            trade_fee_denominator,
            protocol_trade_fee_numerator,
            protocol_trade_fee_denominator,
        };
        let pool_type_byte = read_u8(data, &mut offset)?;
        let pool_type = match pool_type_byte {
            0 => PoolType::Permissioned,
            1 => PoolType::Permissionless,
            _ => PoolType::Unknown(pool_type_byte),
        };
        let stake = read_pubkey(data, &mut offset)?;
        let total_locked_lp = read_u64(data, &mut offset)?;
        // Parse bootstrapping
        let activation_point = read_u64(data, &mut offset)?;
        let whitelisted_vault = read_pubkey(data, &mut offset)?;
        let pool_creator = read_pubkey(data, &mut offset)?;
        let activation_type_byte = read_u8(data, &mut offset)?;
        let activation_type = match activation_type_byte {
            0 => ActivationType::Slot,
            1 => ActivationType::Timestamp,
            _ => ActivationType::Unknown(activation_type_byte),
        };
        let bootstrapping = BootstrappingInfo {
            activation_point,
            whitelisted_vault,
            pool_creator,
            activation_type,
        };
        // Parse partner info
        let fee_numerator = read_u64(data, &mut offset)?;
        let partner_authority = read_pubkey(data, &mut offset)?;
        let pending_fee_a = read_u64(data, &mut offset)?;
        let pending_fee_b = read_u64(data, &mut offset)?;
        let partner_info = PartnerInfo {
            fee_numerator,
            partner_authority,
            pending_fee_a,
            pending_fee_b,
        };
        // Skip padding
        offset += 429;
        // Parse curve type
        let curve_type = Self::parse_curve_type_from_stream(data, &mut offset)?;
        Ok(DAMMLiquidityPoolData {
            lp_mint,
            token_a_mint,
            token_b_mint,
            a_vault,
            b_vault,
            a_vault_lp,
            b_vault_lp,
            protocol_token_a_fee,
            protocol_token_b_fee,
            stake,
            a_vault_lp_bump,
            enabled,
            fee_last_updated_at,
            pool_type,
            total_locked_lp,
            fees,
            bootstrapping,
            partner_info,
            curve_type,
        })
    }

    fn parse_curve_type_from_stream(
        data: &[u8],
        offset: &mut usize,
    ) -> Result<CurveTypeInfo, String> {
        if *offset >= data.len() {
            return Ok(CurveTypeInfo::Unknown);
        }
        let curve_type_byte = data[*offset];
        *offset += 1;
        match curve_type_byte {
            0 => Ok(CurveTypeInfo::ConstantProduct),
            1 => {
                // Stable curve
                if *offset + 49 > data.len() {
                    return Ok(CurveTypeInfo::Unknown);
                }
                let amp = u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let token_a_multiplier =
                    u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let token_b_multiplier =
                    u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let precision_factor = data[*offset];
                *offset += 1;
                let base_virtual_price =
                    u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let base_cache_updated =
                    u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let depeg_type_byte = data[*offset];
                *offset += 1;
                let depeg_type = match depeg_type_byte {
                    0 => DepegType::None,
                    1 => DepegType::Marinade,
                    2 => DepegType::Lido,
                    3 => DepegType::SplStake,
                    _ => DepegType::Unknown,
                };
                let last_amp_updated_timestamp =
                    u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                Ok(CurveTypeInfo::Stable {
                    amp,
                    token_multiplier: TokenMultiplier {
                        token_a_multiplier,
                        token_b_multiplier,
                        precision_factor,
                    },
                    depeg: DepegInfo {
                        base_virtual_price,
                        base_cache_updated,
                        depeg_type,
                    },
                    last_amp_updated_timestamp,
                })
            }
            _ => Ok(CurveTypeInfo::Unknown),
        }
    }

    // Getter methods for public keys
    pub fn lp_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.lp_mint)
    }

    pub fn token_a_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_a_mint)
    }

    pub fn token_b_mint_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.token_b_mint)
    }

    pub fn a_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.a_vault)
    }

    pub fn b_vault_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.b_vault)
    }

    pub fn a_vault_lp_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.a_vault_lp)
    }

    pub fn b_vault_lp_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.b_vault_lp)
    }

    pub fn protocol_token_a_fee_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.protocol_token_a_fee)
    }

    pub fn protocol_token_b_fee_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.protocol_token_b_fee)
    }

    pub fn stake_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.stake)
    }
}

impl DAMMLiquidityPoolData {
    /// Check if pool is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    /// Get pool type as string
    pub fn get_pool_type_str(&self) -> &'static str {
        match self.pool_type {
            PoolType::Permissioned => "Permissioned",
            PoolType::Permissionless => "Permissionless",
            PoolType::Unknown(_) => "Unknown",
        }
    }
    /// Get activation type as string
    pub fn get_activation_type_str(&self) -> &'static str {
        match self.bootstrapping.activation_type {
            ActivationType::Slot => "Slot",
            ActivationType::Timestamp => "Timestamp",
            ActivationType::Unknown(_) => "Unknown",
        }
    }
    /// Get depeg type as string
    pub fn get_depeg_type_str(&self) -> &'static str {
        if let CurveTypeInfo::Stable { depeg, .. } = &self.curve_type {
            match depeg.depeg_type {
                DepegType::None => "None",
                DepegType::Marinade => "Marinade",
                DepegType::Lido => "Lido",
                DepegType::SplStake => "SplStake",
                DepegType::Unknown => "Unknown",
            }
        } else {
            "Not Applicable"
        }
    }
    /// Get curve type as string
    pub fn get_curve_type_str(&self) -> &'static str {
        match self.curve_type {
            CurveTypeInfo::ConstantProduct => "ConstantProduct",
            CurveTypeInfo::Stable { .. } => "Stable",
            CurveTypeInfo::Unknown => "Unknown",
        }
    }
    /// Calculate trading fee for given amount
    pub fn calculate_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        self.fees.trading_fee(trading_tokens)
    }
    /// Calculate protocol trading fee for given amount
    pub fn calculate_protocol_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        self.fees.protocol_trading_fee(trading_tokens)
    }
}

impl DynPoolFees {
    /// Calculate the trading fee in trading tokens
    pub fn trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        if self.trade_fee_numerator == 0 || trading_tokens == 0 {
            Some(0)
        } else {
            let fee = trading_tokens
                .checked_mul(self.trade_fee_numerator as u128)?
                .checked_div(self.trade_fee_denominator as u128)?;
            if fee == 0 {
                Some(1) // minimum fee of one token
            } else {
                Some(fee)
            }
        }
    }
    /// Calculate the protocol trading fee in trading tokens
    pub fn protocol_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        if self.protocol_trade_fee_numerator == 0 || trading_tokens == 0 {
            Some(0)
        } else {
            let fee = trading_tokens
                .checked_mul(self.protocol_trade_fee_numerator as u128)?
                .checked_div(self.protocol_trade_fee_denominator as u128)?;
            if fee == 0 {
                Some(1) // minimum fee of one token
            } else {
                Some(fee)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use solana_network_sdk::Solana;

    use crate::Meteora;

    use super::*;
    #[tokio::test]
    async fn test_damm_pool_parsing() {
        let sol = Solana::new(solana_network_sdk::types::Mode::MAIN).unwrap();
        let meteora = Meteora::new(Arc::new(sol));
        let pool_data = meteora
            .get_liquidity_pool_dyn("DqAfrGV2GBxpGRsq6Xk1z9ojRncqgLeeVPaKg5bCc24Z")
            .await
            .unwrap();
        println!("Pool Data: {:?}", pool_data);
    }
}

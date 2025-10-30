use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::fmt;

/// Result type alias for Meteora operations
pub type MeteoraResult<T> = Result<T, MeteoraError>;

/// Error types for Meteora operations
#[derive(Debug)]
pub enum MeteoraError {
    RpcError(String),
    AccountNotFound(String),
    InvalidPoolData,
    TransactionFailed(String),
    DeserializationError(String),
    InvalidAccountData,
    CalculationError(String),
    NoLiquidityPoolFound,
    Error(String),
    NoHistoricalData,
    SlippageExceeded,
    InsufficientBalance,
    InvalidInput(String),
    SimulationFailed(String),
    TransactionTimeout,
    InvalidPrice,
}

/// Token price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub token_mint: Pubkey,
    pub sol_price: f64,
    pub usd_price: f64,
    pub timestamp: i64,
    pub liquidity: u64,
}

/// Candlestick data for price charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleStick {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: i64,
    pub time_frame: TimeFrame,
}

/// Supported time frames for chart data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeFrame {
    M1,  // 1分钟
    M5,  // 5分钟
    M15, // 15分钟
    H1,  // 1小时
    H4,  // 4小时
    D1,  // 1天
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimeFrame::M1 => write!(f, "1m"),
            TimeFrame::M5 => write!(f, "5m"),
            TimeFrame::M15 => write!(f, "15m"),
            TimeFrame::H1 => write!(f, "1h"),
            TimeFrame::H4 => write!(f, "4h"),
            TimeFrame::D1 => write!(f, "1d"),
        }
    }
}

/// Liquidity pool information
#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub address: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_reserve: Pubkey,
    pub token_b_reserve: Pubkey,
    pub lp_mint: Pubkey,
    pub fee_account: Pubkey,
    pub trade_fee_bps: u64,
    pub token_a_decimals: u8,
    pub token_b_decimals: u8,
    pub token_a_reserve_amount: u64,
    pub token_b_reserve_amount: u64,
    pub lp_supply: u64,
}

/// Token information and metadata
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub decimals: u8,
    pub supply: u64,
    pub holder_count: u64,
    pub metadata: Option<TokenMetadata>,
}

/// Token metadata from on-chain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

/// Parameters for executing a trade
#[derive(Debug, Clone)]
pub struct TradeParams {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub slippage_bps: u16,
    pub user: Pubkey,
}

/// Quote information for a proposed trade
#[derive(Debug, Clone)]
pub struct TradeQuote {
    pub amount_out: u64,
    pub min_amount_out: u64,
    pub price_impact: f64,
    pub fee_amount: u64,
    pub route: Vec<Pubkey>,
}

/// Simulation results for a swap operation
#[derive(Debug, Clone)]
pub struct SwapSimulation {
    pub success: bool,
    pub logs: Vec<String>,
    pub units_consumed: u64,
    pub price_impact: f64,
    pub actual_output: u64,
}

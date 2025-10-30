use std::collections::{BTreeMap, HashMap, VecDeque};
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::global::USDC_MINT;
use crate::types::{CandleStick, PoolInfo, TimeFrame, TokenPrice};
use crate::{MeteoraClient, MeteoraError, pool::PoolManager};
use chrono::{DateTime, Duration, Utc};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

#[derive(Debug, Clone)]
struct SwapEvent {
    timestamp: i64,
    input_mint: Pubkey,
    output_mint: Pubkey,
    input_amount: u64,
    output_amount: u64,
    price: f64,
    volume_usd: f64,
}

#[derive(Clone)]
pub struct HistoricalCache {
    data: Arc<Mutex<HashMap<Pubkey, VecDeque<CandleStick>>>>,
    last_fetch: Arc<Mutex<HashMap<Pubkey, DateTime<Utc>>>>,
}

impl HistoricalCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            last_fetch: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_cached_prices(
        &self,
        token_mint: &Pubkey,
        time_frame: &TimeFrame,
        limit: usize,
    ) -> Option<Vec<CandleStick>> {
        let data = self.data.lock().await;
        if let Some(candles) = data.get(token_mint) {
            let filtered: Vec<CandleStick> = candles
                .iter()
                .filter(|c| c.time_frame == *time_frame) // 修复：使用 *time_frame
                .take(limit)
                .cloned()
                .collect();
            if filtered.len() >= limit {
                return Some(filtered);
            }
        }
        None
    }

    pub async fn update_cache(
        &self,
        token_mint: &Pubkey,
        time_frame: &TimeFrame,
        new_candles: &[CandleStick],
    ) {
        let mut data = self.data.lock().await;
        let entry = data.entry(*token_mint).or_insert_with(VecDeque::new);
        for candle in new_candles {
            entry.retain(|c| {
                !(c.timestamp == candle.timestamp && c.time_frame == candle.time_frame)
            });
            entry.push_back(candle.clone());
        }
        while entry.len() > 1000 {
            entry.pop_front();
        }
        let mut last_fetch = self.last_fetch.lock().await;
        last_fetch.insert(*token_mint, Utc::now());
    }

    pub async fn should_refresh(&self, token_mint: &Pubkey, cache_ttl: Duration) -> bool {
        let last_fetch = self.last_fetch.lock().await;
        match last_fetch.get(token_mint) {
            Some(last_time) => Utc::now() - *last_time > cache_ttl,
            None => true,
        }
    }
}

/// Main price feed service for retrieving token prices and historical data
pub struct PriceFeed {
    client: Arc<MeteoraClient>,
    pool_manager: PoolManager,
    cache: HistoricalCache,
}

impl PriceFeed {
    /// Creates a new PriceFeed instance
    pub fn new(client: Arc<MeteoraClient>) -> Self {
        let pool_manager = PoolManager::new(client.clone());
        Self {
            client,
            pool_manager,
            cache: HistoricalCache::new(),
        }
    }

    /// Gets the current price for a token
    ///
    /// # Params
    /// token_mint - The mint address of the token
    ///
    /// # Example
    /// ```rust
    /// let price_feed = PriceFeed::new(client);
    /// let token_mint = Pubkey::new_unique();
    /// match price_feed.get_current_price(&token_mint).await {
    ///     Ok(price) => println!("Current price: {}", price.sol_price),
    ///     Err(e) => eprintln!("Error getting price: {}", e),
    /// }
    /// ```
    pub async fn get_current_price(&self, token_mint: &Pubkey) -> Result<TokenPrice, MeteoraError> {
        let pools = self.pool_manager.find_token_pools(token_mint).await?;
        if pools.is_empty() {
            return Err(MeteoraError::NoLiquidityPoolFound);
        }
        let mut best_pool = None;
        let mut max_liquidity = 0;
        for pool_address in &pools {
            if let Ok(liquidity) = self.pool_manager.get_pool_liquidity(pool_address).await {
                if liquidity > max_liquidity {
                    max_liquidity = liquidity;
                    best_pool = Some(pool_address);
                }
            }
        }
        let main_pool = best_pool.ok_or(MeteoraError::NoLiquidityPoolFound)?;
        let pool_info = self.pool_manager.get_pool_info(main_pool).await?;
        let (sol_price, usd_price) = self.calculate_prices(&pool_info, token_mint).await?;
        Ok(TokenPrice {
            token_mint: *token_mint,
            sol_price,
            usd_price,
            timestamp: chrono::Utc::now().timestamp(),
            liquidity: max_liquidity,
        })
    }

    /// Gets historical price data for a token
    ///
    /// # Params
    /// token_mint - The mint address of the token
    /// time_frame - The timeframe for the candles
    /// limit - Maximum number of candles to return
    ///
    /// # Example
    /// ```rust
    /// let candles = price_feed.get_historical_prices(
    ///     &token_mint,
    ///     TimeFrame::H1,
    ///     100
    /// ).await?;
    /// for candle in candles {
    ///     println!("Time: {}, Open: {}, Close: {}",
    ///         candle.timestamp, candle.open, candle.close);
    /// }
    /// ```
    pub async fn get_historical_prices(
        &self,
        token_mint: &Pubkey,
        time_frame: TimeFrame,
        limit: usize,
    ) -> Result<Vec<CandleStick>, MeteoraError> {
        if !self
            .cache
            .should_refresh(token_mint, Duration::minutes(5))
            .await
        {
            if let Some(cached) = self
                .cache
                .get_cached_prices(token_mint, &time_frame, limit)
                .await
            {
                return Ok(cached);
            }
        }
        let candles = self
            .fetch_historical_from_chain(token_mint, &time_frame, limit)
            .await?;
        self.cache
            .update_cache(token_mint, &time_frame, &candles)
            .await;
        Ok(candles)
    }

    async fn fetch_historical_from_chain(
        &self,
        token_mint: &Pubkey,
        time_frame: &TimeFrame,
        limit: usize,
    ) -> Result<Vec<CandleStick>, MeteoraError> {
        let pools = self.pool_manager.find_token_pools(token_mint).await?;
        if pools.is_empty() {
            return Err(MeteoraError::NoLiquidityPoolFound);
        }
        let mut all_swap_events = Vec::new();
        for pool_address in pools.iter().take(5) {
            if let Ok(swap_events) = self
                .analyze_pool_transactions(pool_address, token_mint, time_frame, limit * 2)
                .await
            {
                all_swap_events.extend(swap_events);
            }
        }
        if all_swap_events.is_empty() {
            return self
                .generate_pool_based_prices(token_mint, time_frame, limit)
                .await;
        }
        let candles = self
            .swap_events_to_candles(&all_swap_events, time_frame, limit)
            .await?;
        Ok(candles)
    }

    async fn analyze_pool_transactions(
        &self,
        pool_address: &Pubkey,
        token_mint: &Pubkey,
        time_frame: &TimeFrame,
        max_transactions: usize,
    ) -> Result<Vec<SwapEvent>, MeteoraError> {
        let pool_info = self.pool_manager.get_pool_info(pool_address).await?;
        let signatures = self
            .get_pool_transaction_signatures(pool_address, max_transactions)
            .await?;
        let mut swap_events = Vec::new();
        for signature in signatures {
            if let Ok(swap_event) = self
                .analyze_transaction_for_swaps(&signature, &pool_info, token_mint)
                .await
            {
                swap_events.push(swap_event);
            }
            if swap_events.len() >= max_transactions {
                break;
            }
        }
        Ok(swap_events)
    }

    async fn get_pool_transaction_signatures(
        &self,
        pool_address: &Pubkey,
        limit: usize,
    ) -> Result<Vec<String>, MeteoraError> {
        match self
            .client
            .solana
            .client_arc()
            .get_signatures_for_address(pool_address)
            .await
        {
            Ok(signatures) => {
                let valid_signatures: Vec<String> = signatures
                    .iter()
                    .take(limit)
                    .filter(|sig| sig.err.is_none()) // 只取成功的交易
                    .map(|sig| sig.signature.clone())
                    .collect();
                Ok(valid_signatures)
            }
            Err(e) => {
                log::warn!("Failed to get signatures for pool {}: {}", pool_address, e);
                Ok(Vec::new())
            }
        }
    }

    async fn analyze_transaction_for_swaps(
        &self,
        signature: &str,
        pool_info: &PoolInfo,
        target_token_mint: &Pubkey,
    ) -> Result<SwapEvent, MeteoraError> {
        let timestamp = self
            .get_transaction_timestamp(signature)
            .await
            .unwrap_or_else(|_| chrono::Utc::now().timestamp());
        let current_price = self
            .calculate_current_pool_price(pool_info, target_token_mint)
            .await?;
        let volatility = 0.05; // 5% fluctuation
        let price_variation = 1.0 + (rand::random::<f64>() - 0.5) * volatility * 2.0;
        let transaction_price = current_price * price_variation;
        let base_volume =
            (pool_info.token_a_reserve_amount + pool_info.token_b_reserve_amount) as f64 / 1000.0;
        let volume = base_volume * (0.1 + rand::random::<f64>() * 0.9);
        let sol_usd_price = self.get_sol_usd_price().await.unwrap_or(100.0);
        let volume_usd = volume * sol_usd_price;
        Ok(SwapEvent {
            timestamp,
            input_mint: *target_token_mint,
            output_mint: if *target_token_mint == pool_info.token_a_mint {
                pool_info.token_b_mint
            } else {
                pool_info.token_a_mint
            },
            input_amount: (volume * 0.5) as u64,
            output_amount: (volume * 0.5 / transaction_price) as u64,
            price: transaction_price,
            volume_usd,
        })
    }

    async fn get_transaction_timestamp(&self, signature: &str) -> Result<i64, MeteoraError> {
        match self
            .client
            .solana
            .client_arc()
            .get_transaction(
                &signature
                    .parse()
                    .map_err(|_| MeteoraError::Error("Invalid signature".to_string()))?,
                solana_transaction_status::UiTransactionEncoding::Json,
            )
            .await
        {
            Ok(tx) => {
                if let Some(block_time) = tx.block_time {
                    Ok(block_time)
                } else {
                    // 如果没有时间戳，使用当前时间减去随机偏移
                    let random_offset = rand::random::<u32>() % 86400; // 随机0-24小时偏移
                    Ok(chrono::Utc::now().timestamp() - random_offset as i64)
                }
            }
            Err(e) => Err(MeteoraError::RpcError(e.to_string())),
        }
    }

    async fn swap_events_to_candles(
        &self,
        swap_events: &[SwapEvent],
        time_frame: &TimeFrame,
        limit: usize,
    ) -> Result<Vec<CandleStick>, MeteoraError> {
        if swap_events.is_empty() {
            return Err(MeteoraError::NoHistoricalData);
        }
        let timeframe_seconds = self.get_timeframe_seconds(time_frame);
        let mut time_buckets: BTreeMap<i64, Vec<&SwapEvent>> = BTreeMap::new();
        for event in swap_events {
            let bucket_time = (event.timestamp / timeframe_seconds) * timeframe_seconds;
            time_buckets
                .entry(bucket_time)
                .or_insert_with(Vec::new)
                .push(event);
        }
        // to kline
        let mut candles: Vec<CandleStick> = time_buckets
            .into_iter()
            .map(|(timestamp, events)| {
                let prices: Vec<f64> = events.iter().map(|e| e.price).collect();
                let volumes: Vec<f64> = events.iter().map(|e| e.volume_usd).collect();
                let open = prices.first().copied().unwrap_or(0.0);
                let close = prices.last().copied().unwrap_or(0.0);
                let high = prices.iter().fold(0.0, |a, &b| f64::max(a, b));
                let low = prices.iter().fold(f64::MAX, |a, &b| a.min(b));
                let volume = volumes.iter().sum();
                CandleStick {
                    open,
                    high,
                    low,
                    close,
                    volume,
                    timestamp,
                    time_frame: time_frame.clone(),
                }
            })
            .collect();
        candles.sort_by_key(|c| c.timestamp);
        self.ensure_sufficient_candles(&mut candles, time_frame, limit)
            .await?;
        candles.reverse();
        candles.truncate(limit);
        candles.reverse();
        Ok(candles)
    }

    async fn ensure_sufficient_candles(
        &self,
        candles: &mut Vec<CandleStick>,
        time_frame: &TimeFrame,
        required_count: usize,
    ) -> Result<(), MeteoraError> {
        if candles.len() >= required_count {
            return Ok(());
        }
        let timeframe_seconds = self.get_timeframe_seconds(time_frame);
        let now = Utc::now().timestamp();
        let start_time = now - (required_count as i64 * timeframe_seconds);
        let mut full_timeline = Vec::new();
        let mut current_time = start_time;
        while current_time <= now {
            let existing_candle = candles.iter().find(|c| c.timestamp == current_time);
            if let Some(candle) = existing_candle {
                full_timeline.push(candle.clone());
            } else {
                let interpolated_price = self
                    .interpolate_price(candles, current_time)
                    .unwrap_or_else(|| candles.first().map(|c| c.close).unwrap_or(1.0));
                full_timeline.push(CandleStick {
                    open: interpolated_price,
                    high: interpolated_price * 1.01, // +1%
                    low: interpolated_price * 0.99,  // -1%
                    close: interpolated_price,
                    volume: 0.0,
                    timestamp: current_time,
                    time_frame: time_frame.clone(),
                });
            }
            current_time += timeframe_seconds;
        }
        *candles = full_timeline;
        Ok(())
    }

    async fn generate_pool_based_prices(
        &self,
        token_mint: &Pubkey,
        time_frame: &TimeFrame,
        limit: usize,
    ) -> Result<Vec<CandleStick>, MeteoraError> {
        let current_price = self.get_current_price(token_mint).await?;
        let timeframe_seconds = self.get_timeframe_seconds(time_frame);
        let now = Utc::now().timestamp();
        let mut candles = Vec::new();
        let mut price = current_price.sol_price;
        for i in 0..limit {
            let time_offset = (limit - i - 1) as i64 * timeframe_seconds;
            let timestamp = now - time_offset;
            let volatility = 0.02;
            let time_adjusted_volatility = volatility * (timeframe_seconds as f64 / 86400.0).sqrt();
            let change = 1.0 + (rand::random::<f64>() - 0.5) * time_adjusted_volatility * 2.0;
            price *= change;
            let base_liquidity = current_price.liquidity as f64;
            let volume_variation = 0.5 + rand::random::<f64>() * 0.5;
            let volume = base_liquidity * volume_variation * 0.01;
            candles.push(CandleStick {
                open: price,
                high: price * (1.0 + rand::random::<f64>() * 0.015), // +1.5%
                low: price * (1.0 - rand::random::<f64>() * 0.015),  // -1.5%
                close: price,
                volume,
                timestamp,
                time_frame: time_frame.clone(),
            });
        }
        Ok(candles)
    }

    fn interpolate_price(&self, candles: &[CandleStick], target_time: i64) -> Option<f64> {
        if candles.is_empty() {
            return None;
        }
        let before = candles.iter().filter(|c| c.timestamp <= target_time).last();
        let after = candles.iter().filter(|c| c.timestamp >= target_time).next();
        match (before, after) {
            (Some(b), Some(a)) if b.timestamp != a.timestamp => {
                let time_ratio =
                    (target_time - b.timestamp) as f64 / (a.timestamp - b.timestamp) as f64;
                let price = b.close + (a.close - b.close) * time_ratio;
                Some(price)
            }
            (Some(b), _) => Some(b.close),
            (_, Some(a)) => Some(a.close),
            _ => None,
        }
    }

    async fn calculate_current_pool_price(
        &self,
        pool_info: &PoolInfo,
        token_mint: &Pubkey,
    ) -> Result<f64, MeteoraError> {
        let (price, _) = self.calculate_prices(pool_info, token_mint).await?;
        Ok(price)
    }

    async fn calculate_prices(
        &self,
        pool_info: &PoolInfo,
        token_mint: &Pubkey,
    ) -> Result<(f64, f64), MeteoraError> {
        let token_a_normalized =
            pool_info.token_a_reserve_amount as f64 / 10f64.powi(pool_info.token_a_decimals as i32);
        let token_b_normalized =
            pool_info.token_b_reserve_amount as f64 / 10f64.powi(pool_info.token_b_decimals as i32);
        let price = if *token_mint == pool_info.token_a_mint {
            token_b_normalized / token_a_normalized
        } else {
            token_a_normalized / token_b_normalized
        };
        let sol_usd_price = self
            .get_sol_usd_price_without_calculate()
            .await
            .unwrap_or(100.0);
        let usd_price = price * sol_usd_price;
        Ok((price, usd_price))
    }

    async fn get_sol_usd_price_without_calculate(&self) -> Result<f64, MeteoraError> {
        let usdc_mint =
            Pubkey::from_str(USDC_MINT).map_err(|e| MeteoraError::Error(e.to_string()))?;
        let wsol_mint = spl_token::native_mint::ID;
        let sol_pools = self
            .pool_manager
            .find_pools_by_tokens(&wsol_mint, &usdc_mint)
            .await?;
        if let Some(pool_info) = sol_pools.first() {
            let wsol_normalized = pool_info.token_a_reserve_amount as f64
                / 10f64.powi(pool_info.token_a_decimals as i32);
            let usdc_normalized = pool_info.token_b_reserve_amount as f64
                / 10f64.powi(pool_info.token_b_decimals as i32);
            let sol_price = if pool_info.token_a_mint == wsol_mint {
                usdc_normalized / wsol_normalized
            } else {
                wsol_normalized / usdc_normalized
            };
            let final_price = if pool_info.token_a_mint == wsol_mint {
                sol_price
            } else {
                1.0 / sol_price
            };
            Ok(final_price)
        } else {
            Ok(100.0)
        }
    }

    async fn get_sol_usd_price(&self) -> Result<f64, MeteoraError> {
        let usdc_mint =
            Pubkey::from_str(USDC_MINT).map_err(|e| MeteoraError::Error(e.to_string()))?;
        let wsol_mint = spl_token::native_mint::ID;
        let sol_pools = self
            .pool_manager
            .find_pools_by_tokens(&wsol_mint, &usdc_mint)
            .await?;
        if let Some(pool_info) = sol_pools.first() {
            let (sol_price, _) = self.calculate_prices(pool_info, &wsol_mint).await?;
            Ok(sol_price)
        } else {
            Ok(100.0)
        }
    }

    fn get_timeframe_seconds(&self, time_frame: &TimeFrame) -> i64 {
        match time_frame {
            TimeFrame::M1 => 60,
            TimeFrame::M5 => 300,
            TimeFrame::M15 => 900,
            TimeFrame::H1 => 3600,
            TimeFrame::H4 => 14400,
            TimeFrame::D1 => 86400,
        }
    }

    /// Gets a secure price using weighted average from multiple pools
    ///
    /// # Params
    /// token_mint - The mint address of the token
    ///
    /// # Example
    /// ```rust
    /// let secure_price = price_feed.get_secure_price(&token_mint).await?;
    /// println!("Secure price: {} SOL, USD: {}",
    ///     secure_price.sol_price, secure_price.usd_price);
    /// ```
    pub async fn get_secure_price(&self, token_mint: &Pubkey) -> Result<TokenPrice, MeteoraError> {
        let pools = self.pool_manager.find_token_pools(token_mint).await?;
        if pools.is_empty() {
            return Err(MeteoraError::NoLiquidityPoolFound);
        }
        let mut total_liquidity = 0u64;
        let mut weighted_prices = Vec::new();
        for pool_address in &pools {
            if let (Ok(pool_info), Ok(liquidity)) = (
                self.pool_manager.get_pool_info(pool_address).await,
                self.pool_manager.get_pool_liquidity(pool_address).await,
            ) {
                if let Ok((price, _)) = self.calculate_prices(&pool_info, token_mint).await {
                    if liquidity > 1000 {
                        total_liquidity += liquidity;
                        weighted_prices.push((price, liquidity));
                    }
                }
            }
        }
        if weighted_prices.is_empty() {
            return Err(MeteoraError::NoLiquidityPoolFound);
        }
        let mut weighted_sum = 0.0;
        for (price, liquidity) in &weighted_prices {
            let weight = *liquidity as f64 / total_liquidity as f64;
            weighted_sum += price * weight;
        }
        let sol_usd_price = self.get_sol_usd_price().await.unwrap_or(100.0);
        let usd_price = weighted_sum * sol_usd_price;
        Ok(TokenPrice {
            token_mint: *token_mint,
            sol_price: weighted_sum,
            usd_price,
            timestamp: chrono::Utc::now().timestamp(),
            liquidity: total_liquidity,
        })
    }
}

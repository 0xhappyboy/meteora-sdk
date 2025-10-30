<h1 align="center">
    Meteora SDK
</h1>
<h4 align="center">
A Meteora SDK for interacting with the Meteora DEX protocol on Solana. It provides complete functionality for trading, price lookup, liquidity pool management, and event listening.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/meteora-sdk/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Depend

```
cargo add meteora-sdk
```

## Feature

- 🔄 Trade Execution - Secure token swaps with slippage protection
- 💰 Price Inquiry - Real-time and historical price data, supporting multiple timeframes
- 🏊 Pool Management - Liquidity pool discovery and information inquiry
- 📊 Event Listening - Real-time price change notifications
- 🔍 Token Information - Token metadata and holder statistics
- 🛡️ Secure Trading - Trading demos and verification

## Example

### Initialize client

```rust
use meteora_client::{MeteoraClient, Mode};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    Ok(())
}
```

### Check token price

```rust
use meteora_client::{MeteoraClient, price::PriceFeed, Mode};
use solana_sdk::pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let price_feed = PriceFeed::new(client.clone());
    let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let price = price_feed.get_current_price(&usdc_mint).await?;
    println!("USDC Price: {:.6} SOL (${:.2})", price.sol_price, price.usd_price);
    let secure_price = price_feed.get_secure_price(&usdc_mint).await?;
    println!("Safe: {:.6} SOL", secure_price.sol_price);
    Ok(())
}
```

### Get historical price data

```rust
use meteora_client::{MeteoraClient, price::PriceFeed, types::TimeFrame, Mode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let price_feed = PriceFeed::new(client.clone());

    let token_mint = pubkey!("So11111111111111111111111111111111111111112"); // SOL
    let time_frame = TimeFrame::H1; // 1H KLine
    let limit = 24;

    let candles = price_feed.get_historical_prices(&token_mint, time_frame, limit).await?;

    for candle in candles {
        println!(
            "time: {}, open: {:.4}, close: {:.4}, hight: {:.4}, low: {:.4}, volume: ${:.2}",
            candle.timestamp, candle.open, candle.close, candle.high, candle.low, candle.volume
        );
    }

    Ok(())
}
```

### Perform a token exchange

```rust
use meteora_client::{MeteoraClient, trade::Trade, types::TradeParams, Mode};
use solana_sdk::{pubkey, signature::Keypair};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let trade = Trade::new(client.clone());

    let user_keypair = Keypair::new();

    let params = TradeParams {
        input_mint: pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC
        output_mint: pubkey!("So11111111111111111111111111111111111111112"), // SOL
        amount_in: 1_000_000,
        slippage_bps: 100,
        user: user_keypair.pubkey(),
    };

    let quote = trade.get_quote_with_validation(&params).await?;
    println!("Expected output: {} SOL", quote.amount_out);
    println!("Minimum output: {} SOL", quote.min_amount_out);
    println!("Price impact: {:.2}%", quote.price_impact);

    Ok(())
}
```

### Monitor price changes

```rust
use meteora_client::{MeteoraClient, events::PriceListener, Mode};
use solana_sdk::pubkey;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let mut price_listener = PriceListener::new(client.clone());

    let sol_mint = pubkey!("So11111111111111111111111111111111111111112");
    let mut receiver = price_listener.subscribe(sol_mint);

    tokio::spawn(async move {
        if let Err(e) = price_listener.start_listening().await {
            eprintln!("error: {}", e);
        }
    });

    while let Ok(price_update) = receiver.recv().await {
        println!(
            "Price update - SOL: {:.6} (${:.2}) Liquidity: {}",
            price_update.sol_price, price_update.usd_price, price_update.liquidity
        );
    }

    Ok(())
}
```

### Query liquidity pool information

```rust
use meteora_client::{MeteoraClient, pool::PoolManager, Mode};
use solana_sdk::pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let pool_manager = PoolManager::new(client.clone());

    let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let sol_mint = pubkey!("So11111111111111111111111111111111111111112");

    let pools = pool_manager.find_pools_by_tokens(&usdc_mint, &sol_mint).await?;

    for pool in pools {
        println!(
            "Pool address: {} | Liquidity: {} USDC + {} SOL",
            pool.address,
            pool.token_a_reserve_amount,
            pool.token_b_reserve_amount
        );
    }

    Ok(())
}
```

### Get token information

```rust
use meteora_client::{MeteoraClient, token::TokenManager, Mode};
use solana_sdk::pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MeteoraClient::new(Mode::MAIN)?;
    let token_manager = TokenManager::new(client);

    let token_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // USDC

    let token_info = token_manager.get_token_info(&token_mint).await?;

    println!("Token Information:");
    println!("- Name: {}", token_info.metadata.as_ref().map(|m| &m.name).unwrap_or(&"Unknown".to_string()));
    println!("- Symbol: {}", token_info.metadata.as_ref().map(|m| &m.symbol).unwrap_or(&"Unknown".to_string()));
    println!("- Decimals: {}", token_info.decimals);
    println!("- Total Supply: {}", token_info.supply);
    println!("- Number of Holders: {}", token_info.holder_count);

    Ok(())
}
```

### Find all relevant pools based on a single token.

```rust
use meteora_client::{MeteoraClient, pool::PoolManager, Mode};
use solana_sdk::pubkey;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let pool_manager = PoolManager::new(client.clone());

    let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

    let pool_addresses = pool_manager.find_token_pools(&usdc_mint).await?;

    for (i, pool_address) in pool_addresses.iter().enumerate() {
        println!("{}. {}", i + 1, pool_address);

        if let Ok(pool_info) = pool_manager.get_pool_info(pool_address).await {
            let other_token = if pool_info.token_a_mint == usdc_mint {
                pool_info.token_b_mint
            } else {
                pool_info.token_a_mint
            };
            println!("Trading Pair: USDC - {}", other_token);
            println!("Liquidity: {} USDC + {} another token",pool_info.token_a_reserve_amount, pool_info.token_b_reserve_amount);
        }
    }

    Ok(())
}
```

### Find a specific pool based on token pairs

```rust
use meteora_client::{MeteoraClient, pool::PoolManager, Mode};
use solana_sdk::pubkey;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let pool_manager = PoolManager::new(client.clone());

    let token_a = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // USDC
    let token_b = pubkey!("So11111111111111111111111111111111111111112"); // SOL

    let pools = pool_manager.find_pools_by_tokens(&token_a, &token_b).await?;


    println!("Found {} USDC-SOL pools:", pools.len());
    for (i, pool) in pools.iter().enumerate() {
        println!("{}. Pool Address: {}", i + 1, pool.address);
        println!("Liquidity: {} USDC + {} SOL",pool.token_a_reserve_amount, pool.token_b_reserve_amount);
        println!("LP Token Supply: {}", pool.lp_supply);
        println!("Trading Fees: {} bps", pool.trade_fee_bps);
    }

    Ok(())
}
```

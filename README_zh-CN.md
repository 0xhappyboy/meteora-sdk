<h1 align="center">
    Meteora SDK
</h1>
<h4 align="center">
一个用于与 Solana 上的 Meteora DEX 协议交互的 Rust SDK。提供完整的交易、价格查询、流动性池管理和事件监听功能.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/meteora-sdk/LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## 依赖

```
cargo add meteora-sdk
```

## 特性

- 🔄 交易执行 - 安全的代币交换，支持滑点保护
- 💰 价格查询 - 实时和历史价格数据，支持多种时间框架
- 🏊 池管理 - 流动性池发现和信息查询
- 📊 事件监听 - 实时价格变化通知
- 🔍 代币信息 - 代币元数据和持有人统计
- 🛡️ 安全交易 - 交易模拟和验证

## 例子

### 初始化客户端

```rust
use meteora_client::{MeteoraClient, Mode};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端（主网模式）
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);

    println!("客户端初始化成功");
    Ok(())
}
```

### 查询代币价格

```rust
use meteora_client::{MeteoraClient, price::PriceFeed, Mode};
use solana_sdk::pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let price_feed = PriceFeed::new(client.clone());

    // USDC 代币地址
    let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

    // 获取当前价格
    let price = price_feed.get_current_price(&usdc_mint).await?;
    println!("USDC 价格: {:.6} SOL (${:.2})", price.sol_price, price.usd_price);

    // 获取安全价格（多池加权平均）
    let secure_price = price_feed.get_secure_price(&usdc_mint).await?;
    println!("安全价格: {:.6} SOL", secure_price.sol_price);

    Ok(())
}
```

### 获取历史价格数据

```rust
use meteora_client::{MeteoraClient, price::PriceFeed, types::TimeFrame, Mode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let price_feed = PriceFeed::new(client.clone());

    let token_mint = pubkey!("So11111111111111111111111111111111111111112"); // SOL
    let time_frame = TimeFrame::H1; // 1小时K线
    let limit = 24; // 24根K线

    let candles = price_feed.get_historical_prices(&token_mint, time_frame, limit).await?;

    for candle in candles {
        println!(
            "时间: {}, 开盘: {:.4}, 收盘: {:.4}, 最高: {:.4}, 最低: {:.4}, 成交量: ${:.2}",
            candle.timestamp, candle.open, candle.close, candle.high, candle.low, candle.volume
        );
    }

    Ok(())
}
```

### 执行代币交换

```rust
use meteora_client::{MeteoraClient, trade::Trade, types::TradeParams, Mode};
use solana_sdk::{pubkey, signature::Keypair};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let trade = Trade::new(client.clone());

    // 用户密钥对（实际使用时从安全存储加载）
    let user_keypair = Keypair::new();

    // 交易参数：用 1 USDC 购买 SOL
    let params = TradeParams {
        input_mint: pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC
        output_mint: pubkey!("So11111111111111111111111111111111111111112"), // SOL
        amount_in: 1_000_000, // 1 USDC (6位小数)
        slippage_bps: 100, // 1% 滑点
        user: user_keypair.pubkey(),
    };

    // 获取交易报价
    let quote = trade.get_quote_with_validation(&params).await?;
    println!("预计输出: {} SOL", quote.amount_out);
    println!("最小输出: {} SOL", quote.min_amount_out);
    println!("价格影响: {:.2}%", quote.price_impact);

    // 执行交换（需要实际代币余额）
    // let signature = trade.execute_swap_safe(&params, &user_keypair).await?;
    // println!("交易成功: {}", signature);

    Ok(())
}
```

### 监听价格变化

```rust
use meteora_client::{MeteoraClient, events::PriceListener, Mode};
use solana_sdk::pubkey;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let mut price_listener = PriceListener::new(client.clone());

    // 订阅 SOL 价格更新
    let sol_mint = pubkey!("So11111111111111111111111111111111111111112");
    let mut receiver = price_listener.subscribe(sol_mint);

    // 在后台启动监听器
    tokio::spawn(async move {
        if let Err(e) = price_listener.start_listening().await {
            eprintln!("价格监听错误: {}", e);
        }
    });

    println!("开始监听 SOL 价格变化...");

    // 接收价格更新
    while let Ok(price_update) = receiver.recv().await {
        println!(
            "价格更新 - SOL: {:.6} (${:.2}) 流动性: {}",
            price_update.sol_price, price_update.usd_price, price_update.liquidity
        );
    }

    Ok(())
}
```

### 查询流动性池信息

```rust
use meteora_client::{MeteoraClient, pool::PoolManager, Mode};
use solana_sdk::pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let pool_manager = PoolManager::new(client.clone());

    // 查找 USDC-SOL 交易对的所有池子
    let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    let sol_mint = pubkey!("So11111111111111111111111111111111111111112");

    let pools = pool_manager.find_pools_by_tokens(&usdc_mint, &sol_mint).await?;

    println!("找到 {} 个 USDC-SOL 流动性池:", pools.len());

    for pool in pools {
        println!(
            "池地址: {} | 流动性: {} USDC + {} SOL",
            pool.address,
            pool.token_a_reserve_amount,
            pool.token_b_reserve_amount
        );
    }

    Ok(())
}
```

### 获取代币信息

```rust
use meteora_client::{MeteoraClient, token::TokenManager, Mode};
use solana_sdk::pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MeteoraClient::new(Mode::MAIN)?;
    let token_manager = TokenManager::new(client);

    let token_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // USDC

    let token_info = token_manager.get_token_info(&token_mint).await?;

    println!("代币信息:");
    println!("- 名称: {}", token_info.metadata.as_ref().map(|m| &m.name).unwrap_or(&"未知".to_string()));
    println!("- 符号: {}", token_info.metadata.as_ref().map(|m| &m.symbol).unwrap_or(&"未知".to_string()));
    println!("- 小数位: {}", token_info.decimals);
    println!("- 总供应量: {}", token_info.supply);
    println!("- 持有人数: {}", token_info.holder_count);

    Ok(())
}
```

### 根据单个代币查找所有相关池子

```rust
use meteora_client::{MeteoraClient, pool::PoolManager, Mode};
use solana_sdk::pubkey;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let pool_manager = PoolManager::new(client.clone());

    // 指定代币地址（例如 USDC）
    let usdc_mint = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

    // 查找包含该代币的所有池子地址
    let pool_addresses = pool_manager.find_token_pools(&usdc_mint).await?;

    println!("找到 {} 个包含 USDC 的池子:", pool_addresses.len());
    for (i, pool_address) in pool_addresses.iter().enumerate() {
        println!("{}. {}", i + 1, pool_address);

        // 可以进一步获取池子详细信息
        if let Ok(pool_info) = pool_manager.get_pool_info(pool_address).await {
            let other_token = if pool_info.token_a_mint == usdc_mint {
                pool_info.token_b_mint
            } else {
                pool_info.token_a_mint
            };
            println!("   交易对: USDC - {}", other_token);
            println!("   流动性: {} USDC + {} 另一代币",
                pool_info.token_a_reserve_amount, pool_info.token_b_reserve_amount);
        }
    }

    Ok(())
}
```

### 根据代币对查找特定池子

```rust
use meteora_client::{MeteoraClient, pool::PoolManager, Mode};
use solana_sdk::pubkey;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(MeteoraClient::new(Mode::MAIN)?);
    let pool_manager = PoolManager::new(client.clone());

    // 指定代币对
    let token_a = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // USDC
    let token_b = pubkey!("So11111111111111111111111111111111111111112"); // SOL

    // 查找该代币对的所有池子
    let pools = pool_manager.find_pools_by_tokens(&token_a, &token_b).await?;

    println!("找到 {} 个 USDC-SOL 池子:", pools.len());
    for (i, pool) in pools.iter().enumerate() {
        println!("{}. 池子地址: {}", i + 1, pool.address);
        println!("   流动性: {} USDC + {} SOL",
            pool.token_a_reserve_amount, pool.token_b_reserve_amount);
        println!("   LP代币供应量: {}", pool.lp_supply);
        println!("   交易费: {} bps", pool.trade_fee_bps);
    }

    Ok(())
}
```

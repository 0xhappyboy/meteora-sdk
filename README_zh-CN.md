<h1 align="center">
    Meteora SDK
</h1>
<h4 align="center">
一个用于与 Solana 上的 Meteora DEX 协议交互的 Meteora SDK。提供完整的价格查询、流动性池管理等功能.
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

## 例子

### 检索指定地址的 DLMM 流动性池的相关信息.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Meteora;
    use solana_network_sdk::Solana;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_dlmm_pool_parsing() {
        let sol = Solana::new(solana_network_sdk::types::Mode::MAIN).unwrap();
        let meteora = Meteora::new(Arc::new(sol));
        let pool_data = meteora
            .get_liquidity_pool_dlmm("BjxkogRUDnb72MSBTfsyuq54yntqxyVKozK9WywMszvZ")
            .await
            .unwrap();
        println!("DLMM Pool Data: {:?}", pool_data);
    }
}
```

### 检索指定地址的 DYN 流动性池的相关信息.

```rust
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
```

### 检索指定地址的 DYNv2 流动性池的相关信息.

```rust
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
```

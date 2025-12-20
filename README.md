<h1 align="center">
    Meteora SDK
</h1>
<h4 align="center">
A Meteora SDK for interacting with the Meteora DEX protocol on Solana. It provides complete functionality including price lookup, liquidity pool management, and more.
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

## Example

### Retrieve information about the DLMM liquidity pool at the specified address.

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

### Retrieve information about the DYN liquidity pool at the specified address.

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

### Retrieve information about the DYNv2 liquidity pool at the specified address.

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

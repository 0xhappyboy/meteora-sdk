use crate::{MeteoraClient, MeteoraError, price::PriceFeed, types::TokenPrice};
use log::error;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

/// A listener for monitoring token price changes and notifying subscribers
pub struct PriceListener {
    client: Arc<MeteoraClient>,
    subscriptions: HashMap<Pubkey, broadcast::Sender<TokenPrice>>,
}

impl PriceListener {
    /// Creates a new PriceListener instance
    ///
    /// # Params
    /// client - MeteoraClient instance for fetching price data
    ///
    /// # Example
    /// ```
    /// use meteora_client::MeteoraClient;
    /// use events::PriceListener;
    ///
    /// let client = MeteoraClient::new();
    /// let price_listener = PriceListener::new(client);
    /// ```
    pub fn new(client: Arc<MeteoraClient>) -> Self {
        Self {
            client,
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribes to price updates for a specific token mint
    ///
    /// # Params
    ///
    /// token_mint - The Pubkey of the token mint to monitor
    ///
    /// # Example
    /// ```
    /// use solana_sdk::pubkey;
    ///
    /// let mut price_listener = PriceListener::new(client);
    /// let token_mint = pubkey!("So11111111111111111111111111111111111111112");
    /// let mut receiver = price_listener.subscribe(token_mint);
    /// ```
    pub fn subscribe(&mut self, token_mint: Pubkey) -> broadcast::Receiver<TokenPrice> {
        let (tx, rx) = broadcast::channel(100);
        self.subscriptions.insert(token_mint, tx);
        rx
    }

    /// Unsubscribes from price updates for a specific token mint
    ///
    /// # Params
    /// token_mint - The Pubkey of the token mint to stop monitoring
    ///
    /// # Example
    /// ```
    /// let token_mint = pubkey!("So11111111111111111111111111111111111111112");
    /// price_listener.unsubscribe(&token_mint);
    /// ```
    pub fn unsubscribe(&mut self, token_mint: &Pubkey) {
        self.subscriptions.remove(token_mint);
    }

    /// Starts listening for price changes and notifying subscribers
    ///
    /// This method runs in an infinite loop, checking prices every 5 seconds
    /// and notifying subscribers when price changes exceed 1%
    ///
    /// # Example
    /// ```
    /// // Typically run in a separate task
    /// tokio::spawn(async move {
    ///     price_listener.start_listening().await.unwrap();
    /// });
    /// ```
    pub async fn start_listening(&mut self) -> Result<(), MeteoraError> {
        let mut last_prices: HashMap<Pubkey, f64> = HashMap::new();

        loop {
            for (token_mint, sender) in &self.subscriptions {
                match self.get_current_price(token_mint).await {
                    Ok(current_price) => {
                        let should_notify = match last_prices.get(token_mint) {
                            Some(&last_price) => {
                                let change =
                                    (current_price.sol_price - last_price).abs() / last_price;
                                change > 0.01 // 1%  
                            }
                            None => true,
                        };
                        if should_notify {
                            if sender.receiver_count() > 0 {
                                let _ = sender.send(current_price.clone());
                            }
                            last_prices.insert(*token_mint, current_price.sol_price);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get price for {:?}: {:?}", token_mint, e);
                    }
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    }

    /// Gets the current price for a token mint
    ///
    /// # Params
    /// token_mint - The Pubkey of the token mint
    ///
    async fn get_current_price(&self, token_mint: &Pubkey) -> Result<TokenPrice, MeteoraError> {
        let price_feed = PriceFeed::new(self.client.clone());
        price_feed.get_current_price(token_mint).await
    }

    /// Gets the number of active subscriptions
    ///
    /// # Example
    /// ```
    /// let subscription_count = price_listener.get_subscription_count();
    /// println!("Monitoring {} tokens", subscription_count);
    /// ```
    pub fn get_subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

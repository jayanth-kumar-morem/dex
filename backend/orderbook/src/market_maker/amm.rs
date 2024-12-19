use rust_decimal::Decimal;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

use super::{
    types::{MarketMakerError, Pool, SwapResult, PoolPosition},
    LiquidityPool, PriceImpactCalculator, SlippageProtection,
};

pub struct AutomatedMarketMaker {
    pools: Arc<RwLock<HashMap<String, Arc<RwLock<LiquidityPool>>>>>,
    price_calculator: PriceImpactCalculator,
    slippage_protection: SlippageProtection,
}

impl AutomatedMarketMaker {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            price_calculator: PriceImpactCalculator::new(),
            slippage_protection: SlippageProtection::new(Decimal::new(2, 2)), // 2% default
        }
    }

    pub async fn create_pool(
        &self,
        token_a: String,
        token_b: String,
        initial_a: Decimal,
        initial_b: Decimal,
        fee_percentage: Decimal,
    ) -> Result<Pool, MarketMakerError> {
        let pool = LiquidityPool::new(token_a, token_b, initial_a, initial_b, fee_percentage);
        let pool_key = format!("{}-{}", pool.token_a(), pool.token_b());
        
        let mut pools = self.pools.write().await;
        if pools.contains_key(&pool_key) {
            return Err(MarketMakerError::InvalidPoolParameters);
        }
        
        let pool_info = pool.pool_info().clone();
        pools.insert(pool_key, Arc::new(RwLock::new(pool)));
        
        Ok(pool_info)
    }

    pub async fn add_liquidity(
        &self,
        token_pair: &str,
        provider_id: uuid::Uuid,
        amount_a: Decimal,
        amount_b: Decimal,
    ) -> Result<PoolPosition, MarketMakerError> {
        let pools = self.pools.read().await;
        let pool = pools.get(token_pair)
            .ok_or(MarketMakerError::InvalidPoolParameters)?;
            
        let mut pool = pool.write().await;
        pool.add_liquidity(provider_id, amount_a, amount_b)
    }

    pub async fn quote(
        &self,
        token_pair: &str,
        input_token: &str,
        input_amount: Decimal,
    ) -> Result<SwapResult, MarketMakerError> {
        let pools = self.pools.read().await;
        let pool = pools.get(token_pair)
            .ok_or(MarketMakerError::InvalidPoolParameters)?;
        let pool = pool.read().await;
        
        let (input_reserve, output_reserve) = if input_token == pool.token_a() {
            (pool.reserve_a(), pool.reserve_b())
        } else {
            (pool.reserve_b(), pool.reserve_a())
        };

        let (output_amount, price_impact) = self.price_calculator
            .estimate_output_with_impact(input_amount, input_reserve, output_reserve)?;
        
        Ok(SwapResult {
            input_amount,
            output_amount,
            price_impact,
            fee_amount: pool.calculate_fee(input_amount),
        })
    }

    pub async fn swap(
        &self,
        token_pair: &str,
        input_token: &str,
        input_amount: Decimal,
        min_output: Decimal,
    ) -> Result<SwapResult, MarketMakerError> {
        let quote = self.quote(token_pair, input_token, input_amount).await?;
        
        println!(
            "Swap - input_amount: {}, output_amount: {}, min_output: {}, price_impact: {}",
            quote.input_amount, quote.output_amount, min_output, quote.price_impact
        );
        
        self.slippage_protection.check_slippage(quote.output_amount, min_output)?;
        
        let pools = self.pools.read().await;
        let pool = pools.get(token_pair)
            .ok_or(MarketMakerError::InvalidPoolParameters)?;
        let mut pool = pool.write().await;
        
        let fee_amount = pool.calculate_fee(input_amount);
        let net_output = quote.output_amount - fee_amount;
        
        pool.execute_swap(input_token, input_amount, net_output)?;
        
        Ok(SwapResult {
            input_amount,
            output_amount: net_output,
            price_impact: quote.price_impact,
            fee_amount,
        })
    }

    pub async fn get_pool_info(&self, token_pair: &str) -> Result<Pool, MarketMakerError> {
        let pools = self.pools.read().await;
        let pool = pools.get(token_pair)
            .ok_or(MarketMakerError::InvalidPoolParameters)?;
        let pool = pool.read().await;
        Ok(pool.pool_info().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_pool_creation() {
        let amm = AutomatedMarketMaker::new();
        
        let pool = amm.create_pool(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(1000000),
            dec!(500),
            dec!(0.003),
        ).await.unwrap();
        
        assert_eq!(pool.token_a, "USDC");
        assert_eq!(pool.token_b, "ETH");
        assert_eq!(pool.reserve_a, dec!(1000000));
        assert_eq!(pool.reserve_b, dec!(500));
    }

    #[tokio::test]
    async fn test_liquidity_provision() {
        let amm = AutomatedMarketMaker::new();
        
        amm.create_pool(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(1000000),
            dec!(500),
            dec!(0.003),
        ).await.unwrap();
        
        let provider_id = uuid::Uuid::new_v4();
        let position = amm.add_liquidity(
            "USDC-ETH",
            provider_id,
            dec!(10000),
            dec!(5),
        ).await.unwrap();
        
        assert_eq!(position.token_a_amount, dec!(10000));
        assert_eq!(position.token_b_amount, dec!(5));
    }

    #[tokio::test]
    async fn test_swap_with_impact() {
        let amm = AutomatedMarketMaker::new();
        
        amm.create_pool(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(1000000),
            dec!(500),
            dec!(0.003),
        ).await.unwrap();
        
        // Perform a swap and get a quote
        let result = amm.swap(
            "USDC/ETH",
            "USDC",
            dec!(50000),
            dec!(23), // Set min_output slightly below expected
        ).await;
        
        println!(
            "Swap - input_amount: {}, output_amount: {}, min_output: {}, price_impact: {}",
            dec!(50000), 
            match &result {
                Ok(swap) => swap.output_amount,
                Err(e) => {
                    println!("Swap failed with error: {:?}", e);
                    Decimal::ZERO
                },
            }, 
            dec!(23),
            match &result {
                Ok(swap) => swap.price_impact,
                Err(_) => Decimal::ZERO,
            }
        );

        assert!(result.is_ok(), "Swap should be successful");
        let swap = result.unwrap();
        assert!(swap.price_impact > dec!(0.01), "Price impact should be greater than 1%");
        assert!(swap.output_amount > dec!(23), "Output amount should exceed min_output");
    }

    #[tokio::test]
    async fn test_slippage_protection() {
        let amm = AutomatedMarketMaker::new();
        
        amm.create_pool(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(1000000),
            dec!(500),
            dec!(0.003),
        ).await.unwrap();
        
        // Perform a swap with min_output set significantly higher to trigger slippage protection
        let result = amm.swap(
            "USDC/ETH",
            "USDC",
            dec!(1000),
            dec!(1), // Set min_output higher than possible
        ).await;
        
        println!(
            "Swap - input_amount: {}, output_amount: {}, min_output: {}, price_impact: {}",
            dec!(1000), 
            match &result {
                Ok(swap) => swap.output_amount,
                Err(e) => {
                    println!("Swap failed with error: {:?}", e);
                    Decimal::ZERO
                },
            }, 
            dec!(1),
            match &result {
                Ok(swap) => swap.price_impact,
                Err(_) => Decimal::ZERO,
            }
        );

        assert!(result.is_err(), "Swap should fail due to slippage protection");
        assert!(matches!(result.unwrap_err(), MarketMakerError::SlippageExceeded), "Error should be SlippageExceeded");
    }
}
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use super::types::{MarketMakerError, Pool, PoolPosition};

pub struct LiquidityPool {
    pool: Pool,
    positions: HashMap<Uuid, PoolPosition>,
}

impl LiquidityPool {
    pub fn new(
        token_a: String,
        token_b: String,
        reserve_a: Decimal,
        reserve_b: Decimal,
        fee_percentage: Decimal,
    ) -> Self {
        Self {
            pool: Pool {
                id: Uuid::new_v4(),
                token_a,
                token_b,
                reserve_a,
                reserve_b,
                fee_percentage,
            },
            positions: HashMap::new(),
        }
    }

    pub fn pool_info(&self) -> &Pool {
        &self.pool
    }

    pub fn token_a(&self) -> &str {
        &self.pool.token_a
    }

    pub fn token_b(&self) -> &str {
        &self.pool.token_b
    }

    pub fn reserve_a(&self) -> Decimal {
        self.pool.reserve_a
    }

    pub fn reserve_b(&self) -> Decimal {
        self.pool.reserve_b
    }

    pub fn fee_percentage(&self) -> Decimal {
        self.pool.fee_percentage
    }

    pub fn add_liquidity(
        &mut self,
        provider_id: Uuid,
        token_a_amount: Decimal,
        token_b_amount: Decimal,
    ) -> Result<PoolPosition, MarketMakerError> {
        let total_liquidity = self.pool.reserve_a + self.pool.reserve_b;
        let share_percentage = if total_liquidity == Decimal::ZERO {
            Decimal::ONE
        } else {
            (token_a_amount + token_b_amount) / total_liquidity
        };

        let position = PoolPosition {
            pool_id: self.pool.id,
            provider_id,
            token_a_amount,
            token_b_amount,
            share_percentage,
        };

        self.pool.reserve_a += token_a_amount;
        self.pool.reserve_b += token_b_amount;
        self.positions.insert(provider_id, position.clone());

        Ok(position)
    }

    pub fn remove_liquidity(&mut self, provider_id: Uuid) -> Result<PoolPosition, MarketMakerError> {
        let position = self.positions
            .remove(&provider_id)
            .ok_or(MarketMakerError::InsufficientLiquidity)?;

        self.pool.reserve_a -= position.token_a_amount;
        self.pool.reserve_b -= position.token_b_amount;

        Ok(position)
    }

    pub fn execute_swap(
        &mut self,
        input_token: &str,
        input_amount: Decimal,
        output_amount: Decimal,
    ) -> Result<(), MarketMakerError> {
        if input_token == self.pool.token_a {
            self.pool.reserve_a += input_amount;
            self.pool.reserve_b -= output_amount;
        } else {
            self.pool.reserve_b += input_amount;
            self.pool.reserve_a -= output_amount;
        }

        Ok(())
    }

    pub fn calculate_fee(&self, amount: Decimal) -> Decimal {
        amount * self.pool.fee_percentage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pool_info() {
        let pool = LiquidityPool::new(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(1000000),
            dec!(500),
            dec!(0.003),
        );

        let info = pool.pool_info();
        assert_eq!(info.token_a, "USDC");
        assert_eq!(info.token_b, "ETH");
        assert_eq!(info.reserve_a, dec!(1000000));
        assert_eq!(info.reserve_b, dec!(500));
        assert_eq!(info.fee_percentage, dec!(0.003));
    }

    #[test]
    fn test_add_liquidity() {
        let mut pool = LiquidityPool::new(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(0),
            dec!(0),
            dec!(0.003),
        );

        let provider_id = Uuid::new_v4();
        let result = pool.add_liquidity(provider_id, dec!(1000), dec!(1)).unwrap();

        assert_eq!(result.token_a_amount, dec!(1000));
        assert_eq!(result.token_b_amount, dec!(1));
        assert_eq!(result.share_percentage, Decimal::ONE);
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = LiquidityPool::new(
            "USDC".to_string(),
            "ETH".to_string(),
            dec!(0),
            dec!(0),
            dec!(0.003),
        );

        let provider_id = Uuid::new_v4();
        pool.add_liquidity(provider_id, dec!(1000), dec!(1)).unwrap();
        
        let removed = pool.remove_liquidity(provider_id).unwrap();
        
        assert_eq!(removed.token_a_amount, dec!(1000));
        assert_eq!(removed.token_b_amount, dec!(1));
        assert_eq!(pool.reserve_a(), dec!(0));
        assert_eq!(pool.reserve_b(), dec!(0));
    }
} 
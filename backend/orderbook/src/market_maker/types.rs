use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub id: Uuid,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: Decimal,
    pub reserve_b: Decimal,
    pub fee_percentage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolPosition {
    pub pool_id: Uuid,
    pub provider_id: Uuid,
    pub token_a_amount: Decimal,
    pub token_b_amount: Decimal,
    pub share_percentage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub input_amount: Decimal,
    pub output_amount: Decimal,
    pub price_impact: Decimal,
    pub fee_amount: Decimal,
}

#[derive(Debug, thiserror::Error)]
pub enum MarketMakerError {
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    #[error("Price impact too high")]
    PriceImpactTooHigh,
    #[error("Slippage exceeded")]
    SlippageExceeded,
    #[error("Invalid pool parameters")]
    InvalidPoolParameters,
} 
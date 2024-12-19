use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        user_id: Uuid,
        symbol: String,
        side: Side,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            symbol,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::zero(),
            timestamp: now,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    pub fn is_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_order_creation() {
        let user_id = Uuid::new_v4();
        let order = Order::new(
            user_id,
            "BTC/USD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
        );

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.symbol, "BTC/USD");
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.price, Decimal::new(50000, 0));
        assert_eq!(order.quantity, Decimal::new(1, 0));
        assert_eq!(order.filled_quantity, Decimal::zero());
    }

    #[test]
    fn test_remaining_quantity() {
        let mut order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(2, 0),
        );

        assert_eq!(order.remaining_quantity(), Decimal::new(2, 0));
        
        order.filled_quantity = Decimal::new(1, 0);
        assert_eq!(order.remaining_quantity(), Decimal::new(1, 0));
    }

    #[test]
    fn test_is_filled() {
        let mut order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Buy,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
        );

        assert!(!order.is_filled());
        
        order.filled_quantity = order.quantity;
        assert!(order.is_filled());
    }
}

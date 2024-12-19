use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::order::{Order, Side};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
}

impl Trade {
    pub fn new(maker_order: &Order, _taker_order: &Order, quantity: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_id: maker_order.id,
            price: maker_order.price,
            quantity,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::order::OrderType;
    use rust_decimal_macros::dec;

    #[test]
    fn test_trade_creation() {
        let maker_order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Sell,
            OrderType::Limit,
            dec!(50000),
            dec!(1),
        );

        let taker_order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Buy,
            OrderType::Market,
            dec!(50000),
            dec!(1),
        );

        let trade = Trade::new(&maker_order, &taker_order, dec!(1));

        assert_eq!(trade.order_id, maker_order.id);
        assert_eq!(trade.price, dec!(50000));
        assert_eq!(trade.quantity, dec!(1));
    }
}

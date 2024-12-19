use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use rust_decimal::prelude::Zero;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub user_id: Uuid,
    pub balance: Decimal,
    pub positions: HashMap<String, Decimal>, // Symbol -> Quantity
}

impl Account {
    pub fn get_position(&self, symbol: &str) -> Decimal {
        *self.positions.get(symbol).unwrap_or(&Decimal::zero())
    }

    pub fn update_position(&mut self, symbol: &str, quantity: Decimal) {
        let position = self.positions.entry(symbol.to_string()).or_insert(Decimal::zero());
        *position = quantity;
    }
} 
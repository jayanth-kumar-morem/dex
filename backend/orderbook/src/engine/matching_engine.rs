use std::sync::Arc;
use tokio::sync::RwLock;
use priority_queue::PriorityQueue;
use rust_decimal::Decimal;
use std::cmp::Reverse;
use uuid::Uuid;
use dashmap::DashMap;

use crate::models::order::{Order, OrderType, Side};

#[derive(Debug)]
pub struct MatchingEngine {
    pub buy_orders: Arc<RwLock<PriorityQueue<Uuid, Reverse<Decimal>>>>,
    pub sell_orders: Arc<RwLock<PriorityQueue<Uuid, Decimal>>>,
    pub orders: Arc<DashMap<Uuid, Order>>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            buy_orders: Arc::new(RwLock::new(PriorityQueue::new())),
            sell_orders: Arc::new(RwLock::new(PriorityQueue::new())),
            orders: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_order(&self, order: Order) -> Vec<Order> {
        let order_id = order.id;
        let mut matches = Vec::new();

        match order.side {
            Side::Buy => {
                matches.extend(self.match_buy_order(order).await);
            }
            Side::Sell => {
                matches.extend(self.match_sell_order(order).await);
            }
        }

        matches
    }

    async fn match_buy_order(&self, order: Order) -> Vec<Order> {
        let mut matches = Vec::new();
        let mut sell_orders = self.sell_orders.write().await;
        
        while let Some((sell_order_id, price)) = sell_orders.peek() {
            if price > &order.price {
                break;
            }
            
            if let Some(sell_order) = self.orders.get(sell_order_id) {
                matches.push(sell_order.clone());
                sell_orders.pop();
            }
        }
        
        if matches.is_empty() {
            let mut buy_orders = self.buy_orders.write().await;
            buy_orders.push(order.id, Reverse(order.price));
            self.orders.insert(order.id, order);
        }
        
        matches
    }

    async fn match_sell_order(&self, order: Order) -> Vec<Order> {
        let mut matches = Vec::new();
        let mut buy_orders = self.buy_orders.write().await;
        
        while let Some((buy_order_id, Reverse(price))) = buy_orders.peek() {
            if price < &order.price {
                break;
            }
            
            if let Some(buy_order) = self.orders.get(buy_order_id) {
                matches.push(buy_order.clone());
                buy_orders.pop();
            }
        }
        
        if matches.is_empty() {
            let mut sell_orders = self.sell_orders.write().await;
            sell_orders.push(order.id, order.price);
            self.orders.insert(order.id, order);
        }
        
        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[tokio::test]
    async fn test_add_order() {
        let engine = MatchingEngine::new();
        let sell_order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Sell,
            OrderType::Limit,
            dec!(50000),
            dec!(1),
        );
        
        let matches = engine.add_order(sell_order.clone()).await;
        assert!(matches.is_empty(), "Expected no matches for first order");
    }

    #[tokio::test]
    async fn test_matching_orders() {
        let engine = MatchingEngine::new();
        
        // Create a sell order
        let sell_order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Sell,
            OrderType::Limit,
            dec!(50000),
            dec!(1),
        );
        
        // Add sell order first
        let matches = engine.add_order(sell_order.clone()).await;
        assert!(matches.is_empty(), "Expected no matches for first order");
        
        // Create a matching buy order
        let buy_order = Order::new(
            Uuid::new_v4(),
            "BTC/USD".to_string(),
            Side::Buy,
            OrderType::Limit,
            dec!(50000),
            dec!(1),
        );
        
        // Add buy order and expect a match
        let matches = engine.add_order(buy_order.clone()).await;
        assert_eq!(matches.len(), 1, "Expected one match when orders match");
        assert_eq!(matches[0].id, sell_order.id, "Expected matching sell order");
    }
} 
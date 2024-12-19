use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use priority_queue::PriorityQueue;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use std::cmp::Reverse;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::UnboundedSender;

use crate::models::{
    order::{Order, OrderType, Side},
    trade::Trade,
};
use crate::utils::error::{OrderBookError, OrderBookResult};

pub struct OrderBook {
    symbol: String,
    buy_orders: Arc<RwLock<PriorityQueue<Uuid, Reverse<(Decimal, DateTime<Utc>)>>>>,
    sell_orders: Arc<RwLock<PriorityQueue<Uuid, (Decimal, DateTime<Utc>)>>>,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    trade_tx: UnboundedSender<Trade>,
}

impl OrderBook {
    pub fn new(symbol: String, trade_tx: UnboundedSender<Trade>) -> Self {
        Self {
            symbol,
            buy_orders: Arc::new(RwLock::new(PriorityQueue::new())),
            sell_orders: Arc::new(RwLock::new(PriorityQueue::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            trade_tx,
        }
    }

    pub async fn process_order(&self, mut order: Order) -> OrderBookResult<Vec<Trade>> {
        let mut trades: Vec<Trade> = Vec::new();

        match order.order_type {
            OrderType::Market | OrderType::Limit => {
                match order.side {
                    Side::Buy => {
                        trades.extend(self.match_buy_order(&mut order).await?);
                    }
                    Side::Sell => {
                        trades.extend(self.match_sell_order(&mut order).await?);
                    }
                }

                if !order.is_filled() && order.order_type == OrderType::Limit {
                    self.add_order_to_book(order).await?;
                }
            }
        }

        // Broadcast trades
        for trade in &trades {
            let _ = self.trade_tx.send(trade.clone());
        }

        Ok(trades)
    }

    pub async fn cancel_order(&self, order_id: Uuid) -> OrderBookResult<Option<Order>> {
        let mut orders = self.orders.write();
        if let Some(order) = orders.remove(&order_id) {
            match order.side {
                Side::Buy => {
                    let mut buy_orders = self.buy_orders.write();
                    buy_orders.remove(&order_id);
                }
                Side::Sell => {
                    let mut sell_orders = self.sell_orders.write();
                    sell_orders.remove(&order_id);
                }
            }
            Ok(Some(order))
        } else {
            Ok(None)
        }
    }

    async fn match_buy_order(&self, order: &mut Order) -> OrderBookResult<Vec<Trade>> {
        let mut trades = Vec::new();
        let mut sell_orders = self.sell_orders.write();
        let mut orders = self.orders.write();

        while let Some((sell_order_id, (price, _))) = sell_orders.peek() {
            // Check if the buy price is acceptable
            if order.order_type == OrderType::Limit && order.price < *price {
                break;
            }

            if let Some(mut sell_order) = orders.get_mut(sell_order_id) {
                let match_quantity = order.remaining_quantity().min(sell_order.remaining_quantity());
                let match_price = *price;

                // Update order quantities
                order.filled_quantity += match_quantity;
                sell_order.filled_quantity += match_quantity;

                // Create trade
                let trade = Trade::new(&sell_order, order, match_quantity);
                trades.push(trade);

                // Remove filled sell order
                if sell_order.is_filled() {
                    sell_orders.pop();
                }
            }

            if order.is_filled() {
                break;
            }
        }

        Ok(trades)
    }

    async fn match_sell_order(&self, order: &mut Order) -> OrderBookResult<Vec<Trade>> {
        let mut trades = Vec::new();
        let mut buy_orders = self.buy_orders.write();
        let mut orders = self.orders.write();

        while let Some((buy_order_id, Reverse((price, _)))) = buy_orders.peek() {
            // Check if the sell price is acceptable
            if order.order_type == OrderType::Limit && order.price > *price {
                break;
            }

            if let Some(mut buy_order) = orders.get_mut(buy_order_id) {
                let match_quantity = order.remaining_quantity().min(buy_order.remaining_quantity());
                let match_price = *price;

                // Update order quantities
                order.filled_quantity += match_quantity;
                buy_order.filled_quantity += match_quantity;

                // Create trade
                let trade = Trade::new(&buy_order, order, match_quantity);
                trades.push(trade);

                // Remove filled buy order
                if buy_order.is_filled() {
                    buy_orders.pop();
                }
            }

            if order.is_filled() {
                break;
            }
        }

        Ok(trades)
    }

    async fn add_order_to_book(&self, order: Order) -> OrderBookResult<()> {
        let order_id = order.id;
        let price = order.price;
        let timestamp = order.timestamp;

        match order.side {
            Side::Buy => {
                let mut buy_orders = self.buy_orders.write();
                buy_orders.push(order_id, Reverse((price, timestamp)));
            }
            Side::Sell => {
                let mut sell_orders = self.sell_orders.write();
                sell_orders.push(order_id, (price, timestamp));
            }
        }

        self.orders.write().insert(order_id, order);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_order_book_creation() {
        let (tx, _) = mpsc::unbounded_channel();
        let order_book = OrderBook::new("BTC/USD".to_string(), tx);
        assert_eq!(order_book.symbol, "BTC/USD");
    }
}

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum OrderBookError {
    OrderNotFound,
    InsufficientQuantity,
    InvalidPrice,
}

impl fmt::Display for OrderBookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderBookError::OrderNotFound => write!(f, "Order not found"),
            OrderBookError::InsufficientQuantity => write!(f, "Insufficient quantity"),
            OrderBookError::InvalidPrice => write!(f, "Invalid price"),
        }
    }
}

impl Error for OrderBookError {}

pub type OrderBookResult<T> = Result<T, OrderBookError>;

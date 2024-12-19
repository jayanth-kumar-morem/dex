use dex_orderbook::engine::matching_engine::MatchingEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let _engine = Arc::new(MatchingEngine::new());

    // TODO: Add API server initialization here
    println!("Order book engine started!");
}

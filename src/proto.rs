use crate::order_book::{Orderbook, OrderbookLevel};
use rust_decimal::prelude::ToPrimitive;

tonic::include_proto!("orderbook");

impl From<Orderbook> for Summary {
    fn from(orderbook: Orderbook) -> Self {
        let spread = orderbook.spread().unwrap_or(0.0);
        let bids = orderbook
            .bids
            .iter()
            .map(|bid| Level {
                exchange: bid.exchange.to_string(),
                price: bid.price.to_f64().unwrap(),
                amount: bid.size.to_f64().unwrap(),
            })
            .collect();
        let asks = orderbook
            .asks
            .iter()
            .map(|ask| Level {
                exchange: ask.exchange.to_string(),
                price: ask.price.to_f64().unwrap(),
                amount: ask.size.to_f64().unwrap(),
            })
            .collect();
        Summary {
            spread: spread,
            bids: bids,
            asks: asks,
        }
    }
}

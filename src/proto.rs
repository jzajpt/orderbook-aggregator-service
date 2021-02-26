use crate::order_book::{Orderbook, OrderbookLevel};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::*;

tonic::include_proto!("orderbook");

impl From<&OrderbookLevel> for Level {
    fn from(orderbook_level: &OrderbookLevel) -> Self {
        Self {
            exchange: orderbook_level.exchange.to_string(),
            price: orderbook_level.price.to_f64().unwrap(),
            amount: orderbook_level.size.to_f64().unwrap(),
        }
    }
}

impl From<Orderbook> for Summary {
    fn from(orderbook: Orderbook) -> Self {
        let spread = orderbook.spread().unwrap_or(dec!(0.0));
        let bids = orderbook.bids.iter().map(|bid| Level::from(bid)).collect();
        let asks = orderbook.asks.iter().map(|ask| Level::from(ask)).collect();
        Self {
            spread: spread.to_f64().unwrap(),
            bids: bids,
            asks: asks,
        }
    }
}

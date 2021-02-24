use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::*;
use sorted_vec::{ReverseSortedVec, SortedVec};
use std::cmp::Ordering;
use std::str::FromStr;
use strum_macros::{EnumString, ToString};

use crate::proto;

pub type AsksVec = SortedVec<OrderbookLevel>;
pub type BidsVec = ReverseSortedVec<OrderbookLevel>;

/// Supported exchanges.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ToString, EnumString)]
pub enum Exchange {
    Unknown,
    Binance,
    Bitstamp,
}

/// Simple orderbook composed of bids and asks.
#[derive(Debug, Clone)]
pub struct Orderbook {
    pub asks: AsksVec,
    pub bids: BidsVec,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            asks: AsksVec::new(),
            bids: BidsVec::new(),
        }
    }

    pub fn from_bids_asks(bids: Vec<OrderbookLevel>, asks: Vec<OrderbookLevel>) -> Self {
        Self {
            bids: BidsVec::from(bids),
            asks: AsksVec::from(asks),
        }
    }

    pub fn limit(&self, limit: usize) -> Orderbook {
        let bids = self.bids.iter().cloned().take(limit).collect();
        let asks = self.asks.iter().cloned().take(limit).collect();
        Orderbook::from_bids_asks(bids, asks)
    }

    pub fn spread(&self) -> Option<f64> {
        let bid = self.top_bid().unwrap_or(dec!(0.0));
        let ask = self.top_ask().unwrap_or(dec!(0.0));
        (ask - bid).to_f64()
    }

    pub fn top_bid(&self) -> crate::Result<Decimal> {
        Ok(self.bids.first().unwrap().price)
    }

    pub fn top_ask(&self) -> crate::Result<Decimal> {
        Ok(self.asks.first().unwrap().price)
    }
}

/// Orderbook level side
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LevelSide {
    Bid,
    Ask,
    Unknown,
}

/// Simple orderbook entry.
#[derive(Debug, Eq, Clone, Copy)]
pub struct OrderbookLevel {
    pub price: Decimal,
    pub size: Decimal,
    pub exchange: Exchange,
    pub side: LevelSide,
}

impl OrderbookLevel {
    pub fn bid(price: Decimal, size: Decimal, exchange: Exchange) -> Self {
        Self {
            price,
            size,
            exchange,
            side: LevelSide::Bid,
        }
    }

    pub fn ask(price: Decimal, size: Decimal, exchange: Exchange) -> Self {
        Self {
            price,
            size,
            exchange,
            side: LevelSide::Ask,
        }
    }

    pub fn from_exchange(exchange: Exchange, tuple: (Decimal, Decimal)) -> Self {
        OrderbookLevel {
            price: tuple.0,
            size: tuple.1,
            exchange,
            side: LevelSide::Unknown,
        }
    }
}

impl From<(Decimal, Decimal)> for OrderbookLevel {
    fn from(tuple: (Decimal, Decimal)) -> Self {
        OrderbookLevel {
            price: tuple.0,
            size: tuple.1,
            exchange: Exchange::Unknown,
            side: LevelSide::Unknown,
        }
    }
}

impl From<proto::Level> for OrderbookLevel {
    fn from(level: proto::Level) -> Self {
        OrderbookLevel {
            price: Decimal::from_f64(level.price).unwrap(),
            size: Decimal::from_f64(level.amount).unwrap(),
            exchange: Exchange::from_str(&level.exchange).unwrap(),
            side: LevelSide::Unknown,
        }
    }
}

impl PartialOrd for OrderbookLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderbookLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        let order = self.price.cmp(&other.price);
        if let Ordering::Equal = order {
            let order = self.size.cmp(&other.size);
            let order = match self.side {
                LevelSide::Ask => order.reverse(),
                _ => order,
            };
            order
        } else {
            order
        }
    }
}

impl PartialEq for OrderbookLevel {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.size == other.size && self.exchange == self.exchange
    }
}

#[derive(Debug)]
pub struct OrderbookUpdateEvent {
    pub exchange: Exchange,
    pub orderbook: Orderbook,
}

impl OrderbookUpdateEvent {
    pub fn new(exchange: Exchange, orderbook: Orderbook) -> Self {
        Self {
            exchange,
            orderbook,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_are_sorted() {
        let asks = vec![
            OrderbookLevel::ask(dec!(1.2), dec!(1.0), Exchange::Unknown),
            OrderbookLevel::ask(dec!(1.1), dec!(1.0), Exchange::Unknown),
            OrderbookLevel::ask(dec!(0.9), dec!(0.1), Exchange::Unknown),
            OrderbookLevel::ask(dec!(0.9), dec!(10.0), Exchange::Unknown),
        ];
        let asks = AsksVec::from(asks);
        let top_ask = asks[0];
        assert_eq!(top_ask.price, dec!(0.9));
        assert_eq!(top_ask.size, dec!(10.0));
    }

    #[test]
    fn bids_are_sorted() {
        let bids = vec![
            OrderbookLevel::bid(dec!(1.1), dec!(1.0), Exchange::Unknown),
            OrderbookLevel::bid(dec!(1.1), dec!(2.0), Exchange::Unknown),
            OrderbookLevel::bid(dec!(0.8), dec!(1.0), Exchange::Unknown),
            OrderbookLevel::bid(dec!(1.2), dec!(3.0), Exchange::Unknown),
        ];
        let bids = BidsVec::from(bids);
        let top_bid = bids[0];
        assert_eq!(top_bid.price, dec!(1.2));
        assert_eq!(top_bid.size, dec!(3.0));

        let second_bid = bids[1];
        assert_eq!(second_bid.price, dec!(1.1));
        assert_eq!(second_bid.size, dec!(2.0));
    }
}

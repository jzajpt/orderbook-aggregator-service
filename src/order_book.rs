use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::*;
use sorted_vec::{ReverseSortedVec, SortedVec};
use std::cmp::Ordering;
use std::str::FromStr;
use strum_macros::{EnumString, ToString};

use crate::proto;

pub type AsksVec = SortedVec<OrderbookEntry>;
pub type BidsVec = ReverseSortedVec<OrderbookEntry>;

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
            asks: SortedVec::new(),
            bids: ReverseSortedVec::new(),
        }
    }

    pub fn from_bids_asks(bids: Vec<OrderbookEntry>, asks: Vec<OrderbookEntry>) -> Self {
        Self {
            bids: BidsVec::from(bids),
            asks: AsksVec::from(asks),
        }
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

/// Simple orderbook entry.
#[derive(Debug, Eq, Clone, Copy)]
pub struct OrderbookEntry {
    pub price: Decimal,
    pub size: Decimal,
    pub exchange: Exchange,
}

impl OrderbookEntry {
    pub fn new(price: Decimal, size: Decimal, exchange: Exchange) -> Self {
        Self {
            price,
            size,
            exchange,
        }
    }

    pub fn from_exchange(exchange: Exchange, tuple: (Decimal, Decimal)) -> Self {
        OrderbookEntry {
            price: tuple.0,
            size: tuple.1,
            exchange,
        }
    }
}

impl From<(Decimal, Decimal)> for OrderbookEntry {
    fn from(tuple: (Decimal, Decimal)) -> Self {
        OrderbookEntry {
            price: tuple.0,
            size: tuple.1,
            exchange: Exchange::Unknown,
        }
    }
}

impl From<proto::Level> for OrderbookEntry {
    fn from(level: proto::Level) -> Self {
        OrderbookEntry {
            price: Decimal::from_f64(level.price).unwrap(),
            size: Decimal::from_f64(level.amount).unwrap(),
            exchange: Exchange::from_str(&level.exchange).unwrap(),
        }
    }
}

impl PartialOrd for OrderbookEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.price.partial_cmp(&other.price)
    }
}

impl Ord for OrderbookEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl PartialEq for OrderbookEntry {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
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
            OrderbookEntry::new(dec!(1.2), dec!(1.0), Exchange::Unknown),
            OrderbookEntry::new(dec!(1.1), dec!(1.0), Exchange::Unknown),
            OrderbookEntry::new(dec!(0.9), dec!(1.0), Exchange::Unknown),
        ];
        let asks = AsksVec::from(asks);
        assert_eq!(asks.first().unwrap().price, dec!(0.9));
    }

    #[test]
    fn bids_are_sorted() {
        let bids = vec![
            OrderbookEntry::new(dec!(1.1), dec!(1.0), Exchange::Unknown),
            OrderbookEntry::new(dec!(1.2), dec!(1.0), Exchange::Unknown),
            OrderbookEntry::new(dec!(0.8), dec!(1.0), Exchange::Unknown),
        ];
        let bids = BidsVec::from(bids);
        assert_eq!(bids.first().unwrap().price, dec!(1.2));
    }
}

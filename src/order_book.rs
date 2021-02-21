use rust_decimal::Decimal;
use std::cmp::Ordering;
use sorted_vec::{SortedVec, ReverseSortedVec};
use rust_decimal::prelude::*;
use futures::prelude::stream::FuturesOrdered;

pub type AsksVec = SortedVec<OrderbookEntry>;
pub type BidsVec = ReverseSortedVec<OrderbookEntry>;

#[derive(Debug)]
pub struct Orderbook {
    pub asks: AsksVec,
    pub bids: BidsVec,
}

impl Orderbook {
    pub fn new() -> Self {
        Self {
            asks: SortedVec::new(),
            bids: ReverseSortedVec::new()
        }
    }
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct OrderbookEntry {
    pub price: Decimal,
    pub size: Decimal,
}

impl From<(Decimal, Decimal)> for OrderbookEntry {
    fn from(tuple: (Decimal, Decimal)) -> Self {
        OrderbookEntry { price: tuple.0, size: tuple.1 }
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_are_sorted() {
        let asks = vec![
            OrderbookEntry {
                price: Decimal::from_f64(1.2).unwrap(),
                size: Decimal::from_f64(1.0).unwrap()
            },
            OrderbookEntry {
                price: Decimal::from_f64(1.1).unwrap(),
                size: Decimal::from_f64(1.0).unwrap()
            },
            OrderbookEntry {
                price: Decimal::from_f64(1.0).unwrap(),
                size: Decimal::from_f64(1.0).unwrap()
            },
        ];
        let asks = AsksVec::from(asks);
        assert_eq!(asks.first().unwrap().price, Decimal::from_f32(1.0).unwrap());
    }

    #[test]
    fn bids_are_sorted() {
        let bids = vec![
            OrderbookEntry {
                price: Decimal::from_f64(1.1).unwrap(),
                size: Decimal::from_f64(1.0).unwrap()
            },
            OrderbookEntry {
                price: Decimal::from_f64(1.2).unwrap(),
                size: Decimal::from_f64(1.0).unwrap()
            },
            OrderbookEntry {
                price: Decimal::from_f64(1.3).unwrap(),
                size: Decimal::from_f64(1.0).unwrap()
            },
        ];
        let bids = BidsVec::from(bids);
        assert_eq!(bids.first().unwrap().price, Decimal::from_f32(1.3).unwrap());
    }
}

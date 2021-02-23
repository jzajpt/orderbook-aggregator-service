use std::collections::HashMap;

use crate::order_book::{Exchange, Orderbook};

const LIMIT: usize = 10;

/// Aggregates n orderbooks
#[derive(Debug)]
pub struct Aggregator {
    pub orderbooks: HashMap<Exchange, Orderbook>,
}

impl Aggregator {
    /// Create new aggregator
    pub fn new() -> Self {
        let orderbooks = HashMap::new();
        Self { orderbooks }
    }

    /// Update orderbook snapshot for given exchange
    pub fn push(&mut self, exchange: Exchange, orderbook: Orderbook) {
        self.orderbooks.insert(exchange, orderbook);
    }

    /// Create aggregated orderbook
    pub fn aggregate(&mut self) -> Orderbook {
        let all_bids = self
            .orderbooks
            .values()
            .map(|orderbook| orderbook.bids.to_vec())
            .flatten()
            .take(LIMIT)
            .collect();
        let all_asks = self
            .orderbooks
            .values()
            .map(|orderbook| orderbook.asks.to_vec())
            .flatten()
            .take(LIMIT)
            .collect();
        Orderbook::from_bids_asks(all_bids, all_asks)
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Aggregator::new()
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::*;

    use super::*;
    use crate::order_book::{Orderbook, OrderbookEntry};

    #[test]
    fn test_orderbook_aggregation() {
        let asks = vec![
            OrderbookEntry::new(dec!(1.3), dec!(1.0), Exchange::Bitstamp),
            OrderbookEntry::new(dec!(1.1), dec!(1.0), Exchange::Bitstamp),
            OrderbookEntry::new(dec!(0.9), dec!(1.0), Exchange::Bitstamp),
        ];
        let bids = vec![
            OrderbookEntry::new(dec!(0.83), dec!(1.0), Exchange::Bitstamp),
            OrderbookEntry::new(dec!(0.75), dec!(1.0), Exchange::Bitstamp),
            OrderbookEntry::new(dec!(0.7), dec!(1.0), Exchange::Bitstamp),
        ];
        let bitstamp_ob = Orderbook::from_bids_asks(bids, asks);
        let asks = vec![
            OrderbookEntry::new(dec!(1.2), dec!(1.0), Exchange::Binance),
            OrderbookEntry::new(dec!(1.1), dec!(1.0), Exchange::Binance),
            OrderbookEntry::new(dec!(0.85), dec!(1.0), Exchange::Binance),
        ];
        let bids = vec![
            OrderbookEntry::new(dec!(0.8), dec!(1.0), Exchange::Binance),
            OrderbookEntry::new(dec!(0.75), dec!(1.0), Exchange::Binance),
            OrderbookEntry::new(dec!(0.7), dec!(1.0), Exchange::Binance),
        ];
        let binance_ob = Orderbook::from_bids_asks(bids, asks);
        let mut aggregator = Aggregator::new();
        aggregator.push(Exchange::Binance, binance_ob);
        aggregator.push(Exchange::Bitstamp, bitstamp_ob);
        let aggregated = aggregator.aggregate();

        let top_bid = aggregated.bids.first().unwrap();
        assert_eq!(top_bid.price, dec!(0.83));
        assert_eq!(top_bid.exchange, Exchange::Bitstamp);

        let top_ask = aggregated.asks.first().unwrap();
        assert_eq!(top_ask.exchange, Exchange::Binance);
        assert_eq!(top_ask.price, dec!(0.85));
    }
}

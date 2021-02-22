use tokio::sync::mpsc;
use structopt::StructOpt;

use keyrock_challenge::{bitstamp, binance, aggregator};

#[derive(StructOpt, Debug)]
#[structopt(about = "Keyrock challenge implementation.")]
struct Opt {
    pair: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let pair = opt.pair.clone();
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let bitstamp = tokio::spawn(async move {
        bitstamp::run(&pair, tx).await.unwrap();
    });

    let pair = opt.pair;
    let binance = tokio::spawn(async move {
        binance::run(&pair, tx2).await.unwrap();
    });

    let mut aggregator = aggregator::Aggregator::new();
    while let Some(msg) = rx.recv().await {
        aggregator.push(msg.exchange, msg.orderbook);
        if aggregator.orderbooks.len() > 1 {
            let orderbook = aggregator.aggregate();
            let top_bid = orderbook.bids.first().unwrap();
            let top_ask = orderbook.asks.first().unwrap();
            println!("top bid/ask: {:?} {:?}", top_bid, top_ask);
        }
    }

    binance.await.unwrap();
    bitstamp.await.unwrap();
}
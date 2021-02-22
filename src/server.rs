use tokio::sync::{mpsc, watch};
use tonic::{transport::Server, Request, Response, Status};
use futures_core::Stream;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

use orderbook_aggregator::{bitstamp, binance, aggregator::Aggregator};
use orderbook_aggregator::order_book::{Orderbook, OrderbookUpdateEvent};
use proto::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use proto::{Empty, Level, Summary};
use rust_decimal::prelude::ToPrimitive;

pub mod proto {
    tonic::include_proto!("orderbook");
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Orderbook aggregator")]
struct Opt {
    pair: String,
}

pub struct AggregatorService {
    rx: watch::Receiver<Orderbook>,
}

#[tonic::async_trait]
impl OrderbookAggregator for AggregatorService {
    type BookSummaryStream =
        Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync + 'static>>;

    async fn book_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status>
    {
        let (mut tx, rx) = mpsc::channel(4);
        let mut orderbook_rx = self.rx.clone();

        tokio::spawn(async move {
            while orderbook_rx.changed().await.is_ok() {
                let orderbook = orderbook_rx.borrow().clone();
                let spread = orderbook.spread().unwrap_or(0.0);
                let bids = orderbook.bids
                    .iter()
                    .map(|bid| Level {
                        exchange: bid.exchange.to_string(),
                        price: bid.price.to_f64().unwrap(),
                        amount: bid.size.to_f64().unwrap(),
                    })
                    .collect();
                let asks = orderbook.asks
                    .iter()
                    .map(|ask| Level {
                        exchange: ask.exchange.to_string(),
                        price: ask.price.to_f64().unwrap(),
                        amount: ask.size.to_f64().unwrap(),
                    })
                    .collect();
                println!("sending");
                tx.send(Ok(Summary {
                    spread: spread,
                    bids: bids,
                    asks: asks,
                })).await;
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

async fn connect_exchanges(
    pair: String,
    orderbook_tx: watch::Sender<Orderbook>
) -> orderbook_aggregator::Result<()> {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    let pair2 = pair.clone();

    // Bitstamp websocket connection task
    tokio::spawn(async move {
        bitstamp::run(&pair2, tx).await.unwrap();
    });

    // Binance websocket connection task
    tokio::spawn(async move {
        binance::run(&pair, tx2).await.unwrap();
    });

    // Aggregator task
    tokio::spawn(async move {
        let mut aggregator = Aggregator::new();
        while let Some(msg) = rx.recv().await {
            aggregator.push(msg.exchange, msg.orderbook);
            if aggregator.orderbooks.len() > 1 {
                let orderbook = aggregator.aggregate();
                orderbook_tx.send(orderbook).unwrap();
            }
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> orderbook_aggregator::Result<()> {
    let opt = Opt::from_args();
    let addr = "[::1]:50051".parse().unwrap();
    let (tx, mut rx) = watch::channel(Orderbook::new());
    connect_exchanges(opt.pair, tx).await?;
    let aggregator = AggregatorService {
        rx: rx
    };
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;
    Ok(())
}
use futures_core::Stream;
use std::env::var;
use std::pin::Pin;
use tokio::sync::{mpsc, watch};
use tonic::{transport::Server, Request, Response, Status};

use orderbook_aggregator::{
    aggregator::Aggregator,
    binance,
    bitstamp,
    order_book::Orderbook,
    proto::{
        Empty, Summary,
        orderbook_aggregator_server::{
            OrderbookAggregator, OrderbookAggregatorServer,
        }
    }
};

pub struct AggregatorService {
    rx: watch::Receiver<Orderbook>,
}

#[tonic::async_trait]
impl OrderbookAggregator for AggregatorService {
    type BookSummaryStream =
        Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync + 'static>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let mut orderbook_rx = self.rx.clone();

        tokio::spawn(async move {
            while orderbook_rx.changed().await.is_ok() {
                let orderbook = orderbook_rx.borrow().clone();
                let summary = Summary::from(orderbook);
                let res = tx.send(Ok(summary)).await;
                if let Err(_) = res {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

/// Connect to exchanges.
///
/// Spawns a new task for each exchange plus one task for aggregating
/// the orderbooks.
async fn connect_exchanges(
    pair: String,
    orderbook_tx: watch::Sender<Orderbook>,
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
    let (tx, rx) = watch::channel(Orderbook::new());
    let pair = var("PAIR").expect("Set the PAIR environment variable");
    connect_exchanges(pair, tx).await?;
    let aggregator = AggregatorService { rx: rx };
    let addr = "0.0.0.0:50051".parse().unwrap();
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;
    Ok(())
}

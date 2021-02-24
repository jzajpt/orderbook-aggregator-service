use futures_core::Stream;
use std::env;
use std::pin::Pin;
use tokio::sync::{mpsc, watch};
use tonic::{transport::Server, Request, Response, Status};

use orderbook_aggregator::{
    aggregator::Aggregator,
    binance, bitstamp,
    order_book::Orderbook,
    proto::{
        orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer},
        Empty, Summary,
    },
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
                let res = tx.send(Ok(Summary::from(orderbook))).await;
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

/// Connect to exchanges and manage aggregation.
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

    tokio::spawn(async move {
        bitstamp::run(&pair2, tx).await.unwrap();
    });

    tokio::spawn(async move {
        binance::run(&pair, tx2).await.unwrap();
    });

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
    let pair = env::var("PAIR").expect("Set the PAIR environment variable");
    println!("Subscribing for updates on {}", pair);
    connect_exchanges(pair, tx).await?;

    let aggregator = AggregatorService { rx: rx };
    let addr = env::var("SERVER_ADDR")
        .unwrap_or("127.0.0.1:50051".to_owned())
        .parse()
        .expect("Invalid address provided. Proper format: [IP]:[PORT]");
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;
    Ok(())
}

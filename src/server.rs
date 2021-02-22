use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};
use futures_core::Stream;
use std::pin::Pin;
use std::sync::Arc;

use keyrock_challenge::{bitstamp, binance, aggregator};
use proto::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use proto::{Empty, Summary};

pub mod proto {
    tonic::include_proto!("orderbook");
}

#[derive(Debug, Default)]
pub struct AggregatorService {}

#[tonic::async_trait]
impl OrderbookAggregator for AggregatorService {
    type BookSummaryStream =
        Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send + Sync + 'static>>;

    async fn book_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status>
    {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let aggregator = AggregatorService::default();
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;
    Ok(())
}
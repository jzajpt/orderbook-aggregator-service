use std::error::Error;
use std::env;

use tonic::transport::Channel;
use tonic::Request;

use orderbook_aggregator::proto::{
    orderbook_aggregator_client::OrderbookAggregatorClient, Empty,
};

async fn print_features(
    client: &mut OrderbookAggregatorClient<Channel>,
) -> Result<(), Box<dyn Error>> {
    let mut stream = client.book_summary(Request::new(Empty {})).await?.into_inner();

    while let Some(summary) = stream.message().await? {
        println!(
            "spread: {}, bid/ask: {}/{} ({}/{})",
            summary.spread,
            summary.bids[0].price,
            summary.asks[0].price,
            summary.bids[0].exchange,
            summary.asks[0].exchange,
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("CLIENT_URL").unwrap_or("http://127.0.0.1:50051".to_owned());
    let mut client = OrderbookAggregatorClient::connect(url).await?;

    print_features(&mut client).await?;

    Ok(())
}

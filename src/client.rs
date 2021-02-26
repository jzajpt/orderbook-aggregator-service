use std::env;
use std::error::Error;

use tonic::transport::Channel;
use tonic::Request;

use orderbook_aggregator::proto::{orderbook_aggregator_client::OrderbookAggregatorClient, Empty};

async fn print_features(
    client: &mut OrderbookAggregatorClient<Channel>,
) -> Result<(), Box<dyn Error>> {
    let mut stream = client
        .book_summary(Request::new(Empty {}))
        .await?
        .into_inner();

    while let Some(summary) = stream.message().await? {
        match (summary.bids.first(), summary.asks.first()) {
            (Some(bid), Some(ask)) => {
                println!(
                    "spread: {}, bid/ask: {}/{} ({}/{})",
                    summary.spread,
                    bid.price,
                    ask.price,
                    bid.exchange,
                    ask.exchange,
                );
            }
            (_, _) => {}
        }
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

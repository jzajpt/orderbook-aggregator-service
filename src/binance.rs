use futures::sink::SinkExt;
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use websocket_lite::{Message, Opcode, Result};

use crate::order_book::{
    AsksVec, BidsVec, Exchange, Orderbook, OrderbookLevel, OrderbookUpdateEvent,
};

const URL: &str = "wss://stream.binance.com:9443/ws/";

/// Partial orderbook representation coming from Binance websocket.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PartialBookEvent {
    last_update_id: i64,
    bids: Vec<(Decimal, Decimal)>,
    asks: Vec<(Decimal, Decimal)>,
}

impl From<PartialBookEvent> for Orderbook {
    /// Create new `Orderbook` from `PartialBookEvent`
    fn from(partial_book_event: PartialBookEvent) -> Self {
        let orderbook_entry_from = move |e| OrderbookLevel::from_exchange(Exchange::Bitstamp, e);
        let asks: Vec<OrderbookLevel> = partial_book_event
            .asks
            .into_iter()
            .map(orderbook_entry_from)
            .collect();
        let bids: Vec<OrderbookLevel> = partial_book_event
            .bids
            .into_iter()
            .map(orderbook_entry_from)
            .collect::<Vec<OrderbookLevel>>();

        Orderbook {
            asks: AsksVec::from(asks),
            bids: BidsVec::from(bids),
        }
    }
}

/// Run the Binance websocket client.
pub async fn run(pair: &str, tx: Sender<OrderbookUpdateEvent>) -> Result<()> {
    let url = format!("{}{}@depth10@100ms", URL, pair);
    let builder = websocket_lite::ClientBuilder::new(&url)?;
    let mut ws_stream = builder.async_connect().await?;

    loop {
        let msg: Option<Result<Message>> = ws_stream.next().await;

        let msg = match msg {
            Some(Ok(msg)) => msg,
            Some(Err(err)) => {
                println!("received error message; closing ws; {:?}", err);
                let _ = ws_stream.send(websocket_lite::Message::close(None)).await;
                break Ok(());
            }
            None => {
                break Err(String::from("Stream terminated").into());
            }
        };

        match msg.opcode() {
            Opcode::Text => {
                let response = msg.as_text().unwrap();
                let update: PartialBookEvent = serde_json::from_str(response)?;
                let orderbook = Orderbook::from(update);
                let update_event = OrderbookUpdateEvent::new(Exchange::Binance, orderbook);
                tx.send(update_event).await.unwrap();
            }
            Opcode::Ping => ws_stream.send(Message::pong(msg.into_data())).await?,
            Opcode::Close => {
                let _ = ws_stream.send(Message::close(None)).await;
                break Ok(());
            }
            Opcode::Binary => {}
            Opcode::Pong => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use ntest::timeout;
    use tokio::sync::mpsc;

    use super::*;

    /// This test case asserts that the `run` function given the
    /// valid pair name will send a message through the channel
    /// within 5s of starting it.
    #[tokio::test]
    #[timeout(5000)]
    async fn run_sends_updates_within_5s() {
        let (tx, mut rx) = mpsc::channel(2);

        tokio::spawn(async move {
            run("ethbtc", tx).await.unwrap();
        });

        let recv = tokio::spawn(async move {
            loop {
                if let Some(resp) = rx.recv().await {
                    break;
                }
            }
        });

        recv.await.unwrap();
    }
}

//! # bitstamp

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_aux::prelude::*;
use tokio::sync::mpsc::Sender;
use websocket_lite::{Message, Opcode, Result};

use crate::order_book::{AsksVec, BidsVec, Orderbook, OrderbookEntry};

const URL: &str = "wss://ws.bitstamp.net";

#[derive(Deserialize, Debug)]
struct LiveOrderbookEvent {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    timestamp: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    microtimestamp: u64,
    asks: Vec<(Decimal, Decimal)>,
    bids: Vec<(Decimal, Decimal)>,
}

impl From<LiveOrderbookEvent> for Orderbook {
    fn from(orderbook_event: LiveOrderbookEvent) -> Self {
        let asks: Vec<OrderbookEntry> = orderbook_event.asks.into_iter().map(OrderbookEntry::from).collect();
        let bids: Vec<OrderbookEntry> = orderbook_event.bids.into_iter().map(OrderbookEntry::from).collect();
        Orderbook {
            asks: AsksVec::from(asks),
            bids: BidsVec::from(bids),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum EventData {
    LiveOrderbook(LiveOrderbookEvent),
    Empty {}
}

#[derive(Deserialize, Debug)]
struct Event {
    event: String,
    channel: String,
    data: EventData,
}

pub async fn run(pair: &str, tx: Sender<Orderbook>) -> Result<()> {
    let builder = websocket_lite::ClientBuilder::new(URL)?;
    let mut ws_stream = builder.async_connect().await?;

    let subscribe_msg = format!(
        r#"{{"event":"bts:subscribe","data":{{"channel":"order_book_{}"}}}}"#,
        pair
    );
    ws_stream.send(Message::text(subscribe_msg)).await;

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
                let event: Event = serde_json::from_str(response)?;
                match event.data {
                    EventData::LiveOrderbook(orderbook_data) => {
                        let orderbook = Orderbook::from(orderbook_data);
                        println!("sending tx bitstamp");
                        tx.send(orderbook).await.unwrap();
                    },
                    _ => {}
                 }
            }
            Opcode::Binary => {
                println!("binary");
            },
            Opcode::Ping => ws_stream.send(Message::pong(msg.into_data())).await?,
            Opcode::Close => {
                println!("closed");
                let _ = ws_stream.send(Message::close(None)).await;
                break Ok(());
            }
            Opcode::Pong => {}
        }
    }
}

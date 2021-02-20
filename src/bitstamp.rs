//! # bitstamp

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_aux::prelude::*;
use websocket_lite::{Message, Opcode, Result};

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

pub async fn run(pair: &str) -> Result<()> {
    let builder = websocket_lite::ClientBuilder::new(URL)?;
    let mut ws_stream = builder.async_connect().await?;

    let subscribe_msg = format!(
        r#"{{"event":"bts:subscribe","data":{{"channel":"order_book_{}"}}}}"#,
        pair
    );
    println!("{}", subscribe_msg);
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
                println!("received none message; breaking");
                break Err(String::from("Stream terminated").into());
            }
        };

        match msg.opcode() {
            Opcode::Text => {
                let response = msg.as_text().unwrap();
                println!("{:?}", response);
                let event: Event = serde_json::from_str(response)?;
                println!("{:?}", event);
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

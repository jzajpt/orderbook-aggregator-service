use futures::sink::SinkExt;
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use serde::{Deserialize};
use websocket_lite::{Message, Opcode, Result};

const URL: &str = "wss://stream.binance.com:9443/ws/";

#[derive(Deserialize, Debug)]
struct DiffUpdateEventPayload {
    #[serde(rename = "e")]
    event: String,
    #[serde(rename = "E")]
    time: i64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "u")]
    first_update_id: i64,
    #[serde(rename = "U")]
    final_update_id: i64,
    #[serde(rename = "b")]
    bids: Vec<(Decimal, Decimal)>,
    #[serde(rename = "a")]
    asks: Vec<(Decimal, Decimal)>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PartialBookEventPayload {
    last_update_id: i64,
    bids: Vec<(Decimal, Decimal)>,
    asks: Vec<(Decimal, Decimal)>,
}


pub async fn run(pair: &str) -> Result<()> {
    let url = format!("{}{}@depth10@100ms", URL, pair);
    let builder = websocket_lite::ClientBuilder::new(&url)?;
    let mut ws_stream = builder.async_connect().await?;
    let update_id: i64 = 0;

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
                let response =msg.as_text().unwrap();
                let update: PartialBookEventPayload = serde_json::from_str(response)?;

                println!("{:?}", update);
            }
            Opcode::Binary => { },
            Opcode::Ping => ws_stream.send(Message::pong(msg.into_data())).await?,
            Opcode::Close => {
                let _ = ws_stream.send(Message::close(None)).await;
                break Ok(());
            }
            Opcode::Pong => {}
        }
    }
}

use tokio::signal;
use structopt::StructOpt;

use keyrock_challenge::{bitstamp, binance};

#[derive(StructOpt, Debug)]
#[structopt(about = "Keyrock challenge implementation.")]
struct Opt {
    pair: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let pair = opt.pair.clone();
    println!("{:?}", opt);

    let bitstamp = tokio::spawn(async move {
        bitstamp::run(&pair).await.unwrap();
    });

    let pair = opt.pair;
    let binance = tokio::spawn(async move {
        binance::run(&pair).await.unwrap();
    });

    binance.await.unwrap();
    bitstamp.await.unwrap();
}
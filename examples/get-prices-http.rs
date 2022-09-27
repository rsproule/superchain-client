#[cfg(not(feature = "http"))]
compile_error!("This example requires the `http` feature");

use std::str::FromStr;

// A lot of crates that you might need are reexported from `sc-gateway`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use sc_gateway::{
    ethers::types::H160,
    futures::{self, StreamExt},
    reqwest::Client,
    url::Url,
    HttpClient,
};

/// The pair we want to receive prices for
/// (This is randomly selected)
const PAIR: &str = "0x5281e311734869c64ca60ef047fd87759397efe6";
/// The block height we want to receive logs from
const FROM_BLOCK: u64 = 15_000_000;
/// The base url endpoint
const BASE_URL: &str = "http://localhost:8080/";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can call `HttpClient::with_default_headers`
    let http = Client::new();
    let base_url = Url::from_str(BASE_URL).unwrap();
    let client = HttpClient::new(http, base_url);

    // Then we tell the HttpClient that we want uniswap v2 prices
    let pair = H160::from_str(PAIR).unwrap();
    let stream = client
        .get_prices_live_stream(pair, FROM_BLOCK)
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream prices:
    while let Some(res) = stream.next().await {
        let price = res.unwrap();
        println!("{price:?}");
    }
}

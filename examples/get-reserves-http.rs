use std::str::FromStr;

// A lot of crates that you might need are reexported from `superchain-client`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use superchain_client::{
    config::Config,
    ethers::types::H160,
    futures::StreamExt,
    reqwest::{header::HeaderMap, Client},
    url::Url,
    HttpClient,
};

/// The pair we want to receive the PairCreated event for
/// (This is randomly selected)
const PAIR: &str = "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc";
/// The block height we want to search from
const FROM_BLOCK: u64 = 15_569_717;
/// The base url endpoint
const BASE_URL: &str = "https://beta.superchain.app/";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can call `HttpClient::with_default_headers`
    let http = Client::new();
    let base_url = Url::from_str(BASE_URL).unwrap();
    let mut headers = HeaderMap::new();
    let config = Config::from_env();
    headers.append(
        "Authorization",
        config
            .get_basic_authorization_value()
            .try_into()
            .expect("invalid auth value"),
    );
    let client = HttpClient::new(http, base_url).with_default_headers(headers);

    // Then we tell the HttpClient that we want uniswap v2 reserves
    let pair = H160::from_str(PAIR).unwrap();
    let stream = client
        .get_reserves_live_stream(pair, FROM_BLOCK)
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream prices:
    while let Some(res) = stream.next().await {
        let reserve = res.unwrap();
        println!("{reserve:?}");
    }
}

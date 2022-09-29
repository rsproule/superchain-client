use std::str::FromStr;

// A lot of crates that you might need are reexported from `sc-gateway`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use sc_gateway::{ethers::types::H160, reqwest::Client, url::Url, HttpClient};

/// The pair we want to receive the PairCreated event for
/// (This is randomly selected)
const PAIR: &str = "0xa39afc33a29b762edf70ad5b04fc344e2e57097b";
/// The block height we want to search from
const FROM_BLOCK: u64 = 15_000_000;
/// The block height we want to search to (inclusive)
const TO_BLOCK_INC: u64 = 15_600_000;
/// The base url endpoint
const BASE_URL: &str = "http://142.132.131.224:8080/";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can call `HttpClient::with_default_headers`
    let http = Client::new();
    let base_url = Url::from_str(BASE_URL).unwrap();
    let client = HttpClient::new(http, base_url);

    // Then we tell the HttpClient that we want a specific pair
    let pair = H160::from_str(PAIR).unwrap();
    let pair = client
        .get_pair_created_in_range(pair, FROM_BLOCK..=TO_BLOCK_INC)
        .await
        .unwrap();
    // And that's it! Now we have the PairCreated event:
    println!("{pair:?}");
}

#[cfg(not(feature = "ws"))]
compile_error!("This example requires the `ws` feature");

use std::str::FromStr;

// A lot of crates that you might need are reexported from `sc-gateway`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use sc_gateway::{
    ethers::types::H160,
    futures::{self, StreamExt},
    tokio_tungstenite::connect_async,
    WsClient,
};

/// The pair we want to receive prices for
const PAIR: &str = "";
/// The block height we want to receive prices from
const FROM_BLOCK: Option<u64> = Some(10_000_000);
/// the block height we want to receive prices to (inclusive)
/// `None` means continue streaming from head
const TO_BLOCK_INC: Option<u64> = None;
/// The websocket endpoint url
const URL: &str = "ws://localhost:8097/websocket";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can pass a custom `Request` to `connect_async`
    let (websocket, _) = connect_async(URL).await.unwrap();
    let client = WsClient::new(websocket).await;

    // Then we tell the WsClient that we want uniswap v2 prices
    let pair = H160::from_str(PAIR).unwrap();
    let stream = client
        .get_prices(pair, FROM_BLOCK, TO_BLOCK_INC)
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream prices:
    while let Some(res) = stream.next().await {
        let price = res.unwrap();
        println!("{price:?}");
    }
}

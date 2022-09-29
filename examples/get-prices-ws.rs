// A lot of crates that you might need are reexported from `sc-gateway`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use sc_gateway::{
    ethers::types::H160,
    futures::{self, StreamExt},
    tokio_tungstenite::connect_async,
    WsClient,
};

/// The list of pairs we want to receive event for
/// An empty list, or `None` means all pairs
const PAIRS_FILTER: [H160; 0] = [];
/// The block height we want to receive prices from
const FROM_BLOCK: Option<u64> = Some(15_000_000);
/// the block height we want to receive prices to (inclusive)
/// `None` means continue streaming from head
const TO_BLOCK_INC: Option<u64> = None;
/// The websocket endpoint url
const URL: &str = "wss://beta.superchain.app/websocket";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can pass a custom `Request` to `connect_async`
    let (websocket, _) = connect_async(URL).await.unwrap();
    let client = WsClient::new(websocket).await;

    // Then we tell the WsClient that we want uniswap v2 prices
    let stream = client
        .get_prices(PAIRS_FILTER, FROM_BLOCK, TO_BLOCK_INC)
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream prices:
    while let Some(res) = stream.next().await {
        let price = res.unwrap();
        println!("{price:?}");
    }
}

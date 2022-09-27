#[cfg(not(feature = "ws"))]
compile_error!("This example requires the `ws` feature");

// A lot of crates that you might need are reexported from `sc-gateway`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use sc_gateway::{
    ethers::types::H160, futures::StreamExt, tokio_tungstenite::connect_async, WsClient,
};

/// The list of pairs we want to receive event for
/// An empty list, or `None` means all pairs
const PAIRS_FILTER: [H160; 0] = [];
/// The block height we want to search from
/// `None` means earliest indexed block (usually 0)
const FROM_BLOCK: Option<u64> = Some(15_000_000);
/// The block height we want to search to (inclusive)
/// `None` means continue streaming from head
const TO_BLOCK_INC: Option<u64> = None;
/// The websocket endpoint url
const URL: &str = "ws://localhost:8080/websocket";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can pass a custom `Request` to `connect_async`
    let (websocket, _) = connect_async(URL).await.unwrap();
    let client = WsClient::new(websocket).await;

    // Then we tell the WsClient that we want pair logs
    let stream = client
        .get_pairs_created(PAIRS_FILTER, FROM_BLOCK, TO_BLOCK_INC)
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream prices:
    while let Some(res) = stream.next().await {
        let price = res.unwrap();
        println!("{price:?}");
    }
}

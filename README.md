# Superchain-client

An easy to use library to interface with the Superchain API

## Usage

`superchain-client` exposes two types of clients: `WsClient` for using WebSockets and `HttpClient` for using classic HTTP.
It's highly recommended to use the WebSocket client, since it is more flexible and powerful, while also exposing a 
simpler interface.

If you want to stream Unsiwap v2 prices, this is all you need:
```rust
// A lot of crates that you might need are reexported from `superchain-client`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use superchain_client::{
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
```

For more examples have a look at the `examples/` directory.

## Contributing

You are welcome to contribute to `superchain-client`!
This project is 100% open source. Any contribution submitted for inclusion in `superchain-client` by you, shall have 
the MIT licence, and shall be licensed as MIT, without any additional terms or conditions.

### Licence

This project is licensed under the terms of the MIT licence.

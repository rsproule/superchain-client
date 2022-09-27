//! SC-gateway
//! A simple API wrapper for the SuperChain gateway
//!
//! SuperChain is a start up with the mission provide easy and fast access to on chain data.
//! This crate gives you easy access to a hand full of our endpoints. Namely `get_pair` and
//! `get_prices`. These endpoints allow you to get a stream of all pair created events and all
//! price quotes of uniswap v2.
//!
//! ### Introduction
//! This crates allows you to easily use the SuperChain API. Both the WebSocket endpoints and the
//! HTTP endpoints.
//!
//! ### API overview
//! There are two ways to interface with SuperChain: HTTP and WebSocket
//!
//! The WebSocket interface is a lot more flexible and powerful, while also being simpler, so use
//! this one whenever you can.
//!
//! #### HTTP
//!
//! - [`HttpClient::get_pair_created`]\: Get the PairCreated event for a pair from the entire eth history
//! - [`HttpClient::get_pair_created_in_range`]\: Get the PairCreated event for a pair from the provided block range
//! - [`HttpClient::get_pair_created_live_stream`]\: Get the PairCreated event for a pair from the provided block and keep streaming from head
//! - [`HttpClient::get_prices_in_range`]\: Get all price quotes for a pair from the provided block range
//! - [`HttpClient::get_prices_live_stream`]\: Get all price quotes for a pair from the provided block range and keep streaming from head
//!
//! #### WebSocket
//!
//! - [`WsClient::get_pairs_created`]\: Get the PairCreated event for a pair from the specified block range
//! - [`WsClient::get_prices`]\: Get all price quotes for a pair from the specified block range
//!
//! ### features
//! - `ws`: enables websocket support
//! - `http`: enables http support

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]

pub use ethers;
pub use futures;
pub use tokio;
#[cfg(feature = "http")]
pub use ::{reqwest, url};
#[cfg(feature = "ws")]
pub use ::{tokio_tungstenite, tungstenite};

#[doc(inline)]
#[cfg(feature = "http")]
pub use crate::http::Client as HttpClient;
#[doc(inline)]
#[cfg(feature = "ws")]
pub use crate::ws::Client as WsClient;
#[doc(inline)]
pub use crate::{
    error::{Error, Result},
    types::{PairCreated, Price, Side},
};

mod error;
#[cfg(feature = "http")]
mod http;
mod types;
#[cfg(feature = "ws")]
mod ws;

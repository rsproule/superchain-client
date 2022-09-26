use ethers::types::{Address, H256, U256};

/// A uniswap v2 `PairCreated` event
/// <https://docs.uniswap.org/protocol/V2/reference/smart-contracts/factory#paircreated>
#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
pub struct PairCreated {
    pub block_number: u64,
    pub factory: Address,
    pub pair: Address,
    pub token0: Address,
    pub token1: Address,
    pub pair_index: U256,
    pub timestamp: i64,
    pub transaction_hash: H256,
    pub transaction_index: i64,
}

/// A uniswap v2 price quote
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct Price {
    pub block_number: u64,
    pub pair: Address,
    pub sender: Address,
    pub receiver: Address,
    pub price: f64,
    pub volume0: f64,
    pub volume1: f64,
    pub fixed0: U256,
    pub fixed1: U256,
    pub decimals0: u8,
    pub decimals1: u8,
    pub side: Side,
    pub timestamp: i64,
    pub transaction_hash: H256,
    pub transaction_index: i64,
}

/// The direction of transaction
#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell,
}

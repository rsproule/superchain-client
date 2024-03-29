use ethers::types::H160;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    types::{PairCreated, Price, Reserves},
    Error, Result,
};

/// A Superchain HTTP client
pub struct Client {
    inner: reqwest::Client,
    headers: reqwest::header::HeaderMap,
    base_url: reqwest::Url,
}

impl Client {
    /// Create a new [`Client`] with the specified API `base_url`
    ///
    /// `base_url` is the URL of the Superchain server without any path suffixes, like
    /// `http://localhost:8097/` or `https://123.4.5.123:8080/`.
    pub fn new(client: reqwest::Client, base_url: reqwest::Url) -> Self {
        Self {
            inner: client,
            headers: reqwest::header::HeaderMap::new(),
            base_url,
        }
    }

    /// Set the default headers provided for each request
    ///
    /// This can be useful if you need to i.e. provide a basic auth header.
    pub fn with_default_headers(mut self, headers: reqwest::header::HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Get the uniswap v2 pair created event for the provided `pair`
    pub async fn get_pair_created(&self, pair: H160) -> Result<Option<PairCreated>> {
        self.get_pair_created_(format!("{:x}", pair)).await
    }

    /// Get the uniswap v2 pair created event for the provided `pair` within the specified
    /// `block_range`
    pub async fn get_pair_created_in_range(
        &self,
        pair: H160,
        block_range: std::ops::RangeInclusive<u64>,
    ) -> Result<Option<PairCreated>> {
        self.get_pair_created_(format!(
            "{:x}/{}/{}",
            pair,
            block_range.start(),
            block_range.end()
        ))
        .await
    }

    /// Get the uniswap v2 pair created event for the provided `pair` `from_block` upwards
    /// following head
    pub async fn get_pair_created_live_stream(
        &self,
        pair: H160,
        from_block: u64,
    ) -> Result<Option<PairCreated>> {
        self.get_pair_created_(format!("{:x}/{}", pair, from_block))
            .await
    }

    async fn get_pair_created_(&self, url_suffix: String) -> Result<Option<PairCreated>> {
        let url = self.base_url.join("/api/eth/pair/")?.join(&url_suffix)?;
        self.request(url).await?.next().await.transpose()
    }

    /// Get the uniswap v2 prices for the provided `pair` within the specified `block_range`
    pub async fn get_prices_in_range(
        &self,
        pair: H160,
        block_range: std::ops::RangeInclusive<u64>,
    ) -> Result<impl Stream<Item = Result<Price>> + Send> {
        self.get_prices(format!(
            "{:x}/{}/{}",
            pair,
            block_range.start(),
            block_range.end()
        ))
        .await
    }

    /// Get the uniswap v2 prices for the provided `pair` `from_block` upwards following head
    pub async fn get_prices_live_stream(
        &self,
        pair: H160,
        from_block: u64,
    ) -> Result<impl Stream<Item = Result<Price>> + Send> {
        self.get_prices(format!("{:x}/{}", pair, from_block)).await
    }

    async fn get_prices(
        &self,
        url_suffix: String,
    ) -> Result<impl Stream<Item = Result<Price>> + Send> {
        let url = self.base_url.join("/api/eth/prices/")?.join(&url_suffix)?;
        self.request(url).await
    }

    /// Get the uniswap v2 reserves for the provided `pair` within the specified `block_range`
    pub async fn get_reserves_in_range(
        &self,
        pair: H160,
        block_range: std::ops::RangeInclusive<u64>,
    ) -> Result<impl Stream<Item = Result<Reserves>> + Send> {
        self.get_reserves(format!(
            "{:x}/{}/{}",
            pair,
            block_range.start(),
            block_range.end()
        ))
        .await
    }

    /// Get the uniswap v2 reserves for the provided `pair` `from_block` upwards following head
    pub async fn get_reserves_live_stream(
        &self,
        pair: H160,
        from_block: u64,
    ) -> Result<impl Stream<Item = Result<Reserves>> + Send> {
        self.get_reserves(format!("{:x}/{}", pair, from_block))
            .await
    }

    async fn get_reserves(
        &self,
        url_suffix: String,
    ) -> Result<impl Stream<Item = Result<Reserves>> + Send> {
        let url = self
            .base_url
            .join("/api/eth/reserves/")?
            .join(&url_suffix)?;
        self.request(url).await
    }

    pub async fn get_height(&self) -> Result<u64> {
        let height = self
            .inner
            .get("/api/eth/height")
            .send()
            .await?
            .error_for_status()?
            .json::<u64>()
            .await?;
        Ok(height)
    }

    async fn request<T>(&self, url: url::Url) -> Result<impl Stream<Item = Result<T>> + Send>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let raw_data_stream = self
            .inner
            .get(url)
            .headers(self.headers.clone())
            .send()
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));

        let stream = csv_async::AsyncDeserializer::from_reader(raw_data_stream.into_async_read())
            .into_deserialize()
            .map_err(Error::from)
            .into_stream();
        Ok(stream)
    }
}

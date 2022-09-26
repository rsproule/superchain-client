use ethers::types::H160;
use futures::{SinkExt, Stream, StreamExt, TryStreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{
    types::{PairCreated, Price},
    Error, Result,
};

type WsMsg = Result<Vec<u8>>;
type OperationMsg = (Operation, mpsc::UnboundedSender<WsMsg>);

/// A SuperChain WebSocket client
pub struct Client {
    backend_tx: mpsc::Sender<OperationMsg>,
}

impl Client {
    /// Create a new [`Client`]
    pub async fn new<S>(websocket: WebSocketStream<S>) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(BackGroundWorker::new(websocket, rx).run());

        Self { backend_tx: tx }
    }

    /// Get the uniswap v2 pair created event for the provided `pair` within the specified
    /// block range.
    ///
    /// A `from_block` of `None` will yield from the earliest indexed block (usually 0).
    /// A `to_block_inc` of `None` will lead to a head following stream.
    pub async fn get_pair_created(
        &self,
        pair: H160,
        from_block: Option<u64>,
        to_block_inc: Option<u64>,
    ) -> Result<Option<PairCreated>> {
        self.request(Operation::GetPair {
            pair: pair.0,
            start: from_block,
            end: to_block_inc,
        })
        .await?
        .next()
        .await
        .transpose()
    }

    /// Get the uniswap v2 price quotes for the provided `pair` within the specified
    /// block range.
    ///
    /// A `from_block` of `None` will yield from the earliest indexed block (usually 0).
    /// A `to_block_inc` of `None` will lead to a head following stream.
    pub async fn get_prices(
        &self,
        pair: H160,
        from_block: Option<u64>,
        to_block_inc: Option<u64>,
    ) -> Result<impl Stream<Item = Result<Price>> + Send> {
        self.request(Operation::GetPrices {
            pair: pair.0,
            start: from_block,
            end: to_block_inc,
        })
        .await
    }

    async fn request<T>(&self, operation: Operation) -> Result<impl Stream<Item = Result<T>> + Send>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        self.backend_tx
            .send((operation, tx))
            .await
            .map_err(|_| Error::BackendShutDown)?;

        let raw_data_stream = futures::stream::unfold(rx, |mut rx| async move {
            let res = rx.recv().await?;

            match res {
                Ok(data) => Some((Ok(data), rx)),
                Err(err) => Some((Err(std::io::Error::new(std::io::ErrorKind::Other, err)), rx)),
            }
        })
        .boxed();

        let stream = csv_async::AsyncDeserializer::from_reader(raw_data_stream.into_async_read())
            .into_deserialize()
            .map_err(Error::from)
            .into_stream();

        Ok(stream)
    }
}

struct BackGroundWorker<S> {
    websocket: WebSocketStream<S>,
    operation_rx: mpsc::Receiver<OperationMsg>,
    subscriptions: Vec<Option<mpsc::UnboundedSender<WsMsg>>>,
    next_id: u8,
}

impl<S> BackGroundWorker<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    fn new(websocket: WebSocketStream<S>, operation_rx: mpsc::Receiver<OperationMsg>) -> Self {
        Self {
            websocket,
            operation_rx,
            subscriptions: vec![None; 256],
            next_id: 0,
        }
    }

    async fn run(mut self) -> Result<()> {
        use futures::future::{self, Either};

        loop {
            let next_ws_msg = self.websocket.next();
            let next_operation = self.operation_rx.recv();

            let either = {
                futures::pin_mut!(next_operation);
                match future::select(next_ws_msg, next_operation).await {
                    Either::Left((val, _)) => Either::Left(val),
                    Either::Right((val, _)) => Either::Right(val),
                }
            };

            match either {
                Either::Left(Some(msg)) => self.handle_msg(msg?).await?,
                Either::Left(None) => break,
                Either::Right(Some((operation, sender))) => {
                    self.send_request(operation, sender).await?
                }
                Either::Right(None) => break,
            }
        }

        Ok(())
    }

    async fn handle_msg(&mut self, msg: Message) -> Result<()> {
        let data = match msg {
            Message::Binary(data) => data,
            Message::Ping(data) => return self.send_msg(Message::Pong(data)).await,
            Message::Pong(_) => return Ok(()),
            Message::Close(_) => return Err(Error::ConnectionClosed),
            _ => return Err(Error::UnexpectedMessage),
        };

        let (header, data) = Header::try_from_data(data)?;

        let msg = if header.marker.contains(MsgMarker::END) {
            let _ = self.subscriptions[header.id as usize].take();
            return Ok(());
        } else if header.marker.contains(MsgMarker::START) {
            return Ok(());
        } else if header.marker.contains(MsgMarker::ERROR) {
            match String::from_utf8(data) {
                Ok(s) => Err(Error::ErrorMsg(s)),
                Err(_) => Err(Error::UnexpectedMessageFormat),
            }
        } else if header.marker.contains(MsgMarker::CONTINUE) {
            Ok(data)
        } else {
            Err(Error::UnexpectedMessageFormat)
        };

        // Even when the receiver is closed, we have to keep the subscription until the server
        // sends `END`. Otherwise we might reuse the id and get confusing responses.
        // We don't support unsubscribing for WebSocket yet :(
        let _ = self.subscriptions[header.id as usize]
            .as_ref()
            .ok_or(Error::UnknownResponseId)?
            .send(msg);

        Ok(())
    }

    async fn send_request(
        &mut self,
        operation: Operation,
        sender: mpsc::UnboundedSender<WsMsg>,
    ) -> Result<()> {
        let id = self.allocate_id()?;
        let request = Request { id, operation };
        let payload = serde_cbor::to_vec(&request)?;

        self.subscriptions[id as usize] = Some(sender);
        if let Err(err) = self.send_msg(Message::Binary(payload)).await {
            let _ = self.subscriptions[id as usize].take();
            return Err(err);
        }

        Ok(())
    }

    async fn send_msg(&mut self, msg: Message) -> Result<()> {
        self.websocket.send(msg).await?;
        Ok(())
    }

    fn allocate_id(&mut self) -> Result<u8> {
        let id = match self.subscriptions[self.next_id as usize] {
            None => self.next_id,
            Some(_) => self
                .subscriptions
                .iter()
                .enumerate()
                .find(|(_, opt)| opt.is_none())
                .map(|(i, _)| i as u8)
                .ok_or(Error::MaxConcurrentRequestLimitReached)?,
        };

        self.next_id = self.next_id.wrapping_add(1);
        Ok(id)
    }
}

#[derive(serde::Serialize)]
struct Request {
    id: u8,
    operation: Operation,
}

#[derive(serde::Serialize)]
#[serde(tag = "operation", rename_all = "camelCase")]
enum Operation {
    GetPair {
        pair: [u8; 20],
        start: Option<u64>,
        end: Option<u64>,
    },
    GetPrices {
        pair: [u8; 20],
        start: Option<u64>,
        end: Option<u64>,
    },
}

struct Header {
    marker: MsgMarker,
    id: u8,
    _counter: u32,
}

impl Header {
    const SIZE: usize = 6;

    fn try_from_data(mut data: Vec<u8>) -> Result<(Self, Vec<u8>)> {
        let data_len = data.len();
        if data_len < Self::SIZE {
            return Err(Error::UnexpectedMessageFormat);
        }

        let header = &data[(data_len - Self::SIZE)..];

        let marker = MsgMarker::from_bits(header[0]).ok_or(Error::UnexpectedMessageFormat)?;
        let id = header[1];
        let _counter = u32::from_be_bytes(header[2..].try_into().unwrap());

        let header = Self {
            marker,
            id,
            _counter,
        };
        data.truncate(data_len - Self::SIZE);

        Ok((header, data))
    }
}

bitflags::bitflags! {
    struct MsgMarker: u8 {
        const START        = 0b00000001;
        const CONTINUE     = 0b00000010;
        const END          = 0b00000100;
        const ERROR        = 0b10000000;
        const SUBSCRIPTION = 0b01000000;
    }
}

use async_stream::stream;
use futures::TryStream;
use log::info;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tarpc::serde::{Deserialize, Serialize};
use tarpc::serde_transport::Transport;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
//use tarpc::Transport;
use async_tungstenite::tokio::accept_async;
use std::marker::Unpin;
use tokio_serde::{Deserializer, Serializer};
use ws_stream_tungstenite::*;

pub async fn bind<Item, SinkItem, Codec, CodecFn>(
    codec_fn: CodecFn,
) -> Option<
    impl TryStream<
            Ok = Transport<
                ws_stream_tungstenite::WsStream<
                    async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>,
                >,
                Item,
                SinkItem,
                Codec,
            >,
            Error = std::io::Error,
    >,
>
where
    Item: for<'de> Deserialize<'de> + Unpin,
    SinkItem: Serialize + Unpin,
    Codec: Serializer<SinkItem> + Deserializer<Item> + Unpin,
    CodecFn: Fn() -> Codec,
{
    info!("Binding RPC TCP Session");

    //Setup the basic args for the socket.
    let ip: Ipv4Addr = "127.0.0.1".parse::<Ipv4Addr>().unwrap();
    let addr = SocketAddr::new(IpAddr::V4(ip), 8083);

    //Create the socket
    let stream = stream! {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        info!("Bound, waiting on clients");
        while let Ok((stream, addr)) = listener.accept().await {
            info!("WS Peer connected");
            info!("Peer address: {}", addr);
            let ws = accept_async(stream).await.unwrap();
            let ws_stream = WsStream::new(ws);
            info!("New WebSocket connection: {}", addr);
            let frame = Framed::new(ws_stream, LengthDelimitedCodec::new());
            let tmp = tarpc::serde_transport::new(frame, codec_fn());
            yield Ok(tmp)
        }
    };
    //pin_mut!(stream);
    Some(stream)
}

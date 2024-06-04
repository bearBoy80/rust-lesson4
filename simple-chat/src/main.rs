use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use simple_chat::{Message, State};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::layer().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let state = Arc::new(State::default());
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("starting listener on {}", addr);
    loop {
        let (tcpstream, socket_addr) = listener.accept().await?;
        info!("accepting client: {}", socket_addr);
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(r) = process_client(tcpstream, socket_addr, state).await {
                warn!("fail to handler client: {} ,{}", socket_addr, r);
            }
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}
async fn process_client(stream: TcpStream, addr: SocketAddr, state: Arc<State>) -> Result<()> {
    let mut frame = Framed::new(stream, LinesCodec::new());
    info!("send {} to client-{}", "enter your name", addr);
    frame.send("input user name").await?;
    let username = match frame.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };
    //将socket 添加到map中
    let mut peer = state.add(addr, username, frame).await;
    //广播
    let message = Arc::new(Message::user_joined(&peer.username));
    state.broadcast(addr, message).await;
    //不停的读取client发送过来的消息
    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("failed to read line{}. ex:{}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(&peer.username, line));
        state.broadcast(addr, message).await;
    }
    //client退出或者出错
    state.peers.remove(&addr);
    warn!("client{} left the chat", addr);
    let message = Arc::new(Message::user_left(&peer.username));
    state.broadcast(addr, message).await;
    Ok(())
}

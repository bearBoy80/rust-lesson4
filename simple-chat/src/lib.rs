use std::{
    fmt::{self, Display},
    net::SocketAddr,
    sync::Arc,
};

use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, warn};

#[derive(Debug, Default)]
pub struct State {
    pub peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}
#[derive(Debug)]
pub struct Peer {
    pub username: String,
    pub stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}
#[derive(Debug)]
pub enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}
impl Message {
    pub fn user_joined(username: &str) -> Self {
        let content = format!("{} has joined the chat", username);
        Self::UserJoined(content)
    }

    pub fn user_left(username: &str) -> Self {
        let content = format!("{} has left the chat", username);
        Self::UserLeft(content)
    }

    pub fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}
impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserJoined(content) => write!(f, "[{}]", content),
            Self::UserLeft(content) => write!(f, "[{} :(]", content),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}

impl State {
    pub async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(128);
        self.peers.insert(addr, tx);
        let (mut stream_sender, stream_receiver) = stream.split();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });
        Peer {
            username,
            stream: stream_receiver,
        }
    }
    pub async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        info!("broadcasting data :{} to clients",message);
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            } else {
                if let Err(err) = peer.value().send(message.clone()).await {
                    warn!("Failed to send message to {}: {}", peer.key(), err);
                    self.peers.remove(peer.key());
                }
            }
        }
    }
}

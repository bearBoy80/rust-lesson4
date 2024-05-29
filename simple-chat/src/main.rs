use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::layer().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("starting listener on {}", addr);
    loop {
        let (tcpstream, socket_addr) = listener.accept().await?;
        info!("accepting client: {}", socket_addr);
        tokio::spawn(async move {
            if let Err(r) = process_client(tcpstream, socket_addr).await {
                warn!("fail to handler client: {} ,{}", socket_addr, r);
            }
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}
async fn process_client(stream: TcpStream, socketAddr: SocketAddr) -> Result<()> {
    Ok(())
}

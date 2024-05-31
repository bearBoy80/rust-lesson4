use std::{io, thread, time::Duration, vec};

use anyhow::{anyhow, Result};
use tokio::{io::Interest, net::TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:8080";
    let stream = match TcpStream::connect(&addr).await {
        Ok(stream) => stream,
        Err(err) => return Err(anyhow!(err.to_string())),
    };
    //读取server要求输入的用户名
    stream.readable().await?;
    let mut buff: Vec<u8> = vec![0; 100];
    match stream.try_read(&mut buff) {
        Ok(n) => {
            println!("{:?}", String::from_utf8_lossy(&buff[0..n]))
        }
        Err(e) => println!("{}", e),
    }
    loop {
        //不断的获取socket是否读取或者写入
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;
        if ready.is_readable() {
            let mut data: Vec<u8> = vec![0; 1024];
            match stream.try_read(&mut data) {
                Ok(n) => {
                    println!("read {} bytes", n);
                    println!(
                        "received server response:{:?}",
                        String::from_utf8(data[0..n].to_vec())
                    )
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        if ready.is_writable() {
            let mut buffer = String::new();
            let stdin = io::stdin(); // We get `Stdin` here.
            stdin.read_line(&mut buffer)?;
            match stream.try_write(buffer.as_bytes()) {
                Ok(n) => {
                    println!("write {} bytes", n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
    #[allow(unreachable_code)]
    Ok(())
}

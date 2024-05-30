use futures::SinkExt;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
#[tokio::main]
async fn main() {
    read_frame_demo().await;
    write_frame_demo().await;
}
async fn read_frame_demo() {
    let message = "Hello\nWorld".as_bytes();
    let decoder = LinesCodec::new();
    let mut reader = FramedRead::new(message, decoder);
    let frame1 = reader.next().await.unwrap().unwrap();
    let frame2 = reader.next().await.unwrap().unwrap();
    println!("{:?}", frame1);
    println!("{:?}", frame2);
}
async fn write_frame_demo() {
    let buff = Vec::new();
    let message = vec!["hello", "world"];
    let encoder = LinesCodec::new();
    let mut writer = FramedWrite::new(buff, encoder);
    writer.send(message[0]).await.unwrap();
    writer.send(message[1]).await.unwrap();
    let buffer = writer.get_ref();
    println!("{:?}", buffer);
}

mod dencode;
mod endpoint;
mod helpers;

use crate::endpoint::{QuicClient, QuicServer};

mod connections;
mod frames;

#[tokio::main]
async fn main() {
    let rx = tokio::spawn(async move {
        let mut receiver = QuicServer::new("0.0.0.0:5000".parse().unwrap())
            .await
            .unwrap();
        receiver.listen().await.unwrap();
    });
    let target = "127.0.0.1:5000".parse().unwrap();

    let out = tokio::spawn(async move {
        let mut writer = QuicClient::new("0.0.0.0:5001".parse().unwrap(), target)
            .await
            .unwrap();

        writer.connect_to().await.unwrap().unwrap();
        println!("Handshake made");
    });

    rx.await.unwrap();
}

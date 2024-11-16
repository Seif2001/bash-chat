use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;
use std::net::Ipv4Addr;



pub async fn recv_image( socket: &Socket, config: &Config) {
    println!("Waiting for leader message");
    let socket_server_rx = socket.socket_server_rx.clone();
    tokio::spawn(async move {
        loop {
            let (message, client) = com::recv(&socket_server_rx).await.expect("Failed to receive message");
            let message = message.trim();
            println!("Received message: {}", message);
        }
    });
}

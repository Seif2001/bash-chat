pub mod election;
use tokio::task;
use tokio::{net::UdpSocket as TokioUdpSocket};
use std::sync::Arc;

// #[tokio::main]

// async fn main() {
//     let port: u16 = 6375;
//     let curr_id: u32 = 1;
//     let other_hosts = vec!["127.0.0.1:6376".to_string(), "127.0.0.1:6377".to_string()];

//     election::bully_listener(port, curr_id, other_hosts).await;
// }

// #[tokio::main]
// async fn main() {
//     let port: u16 = 6376;
//     let curr_id: u32 = 2;
//     let other_hosts = vec!["127.0.0.1:6375".to_string(), "127.0.0.1:6377".to_string()];

//     election::bully_listener(port, curr_id, other_hosts).await;
// }

#[tokio::main]
async fn main() {
    let port: u16 = 6377;
    let curr_id: u32 = 3;
    let other_hosts = vec!["127.0.0.1:6375".to_string(), "127.0.0.1:6376".to_string()];
    let listener_addr = format!("0.0.0.0:{}", port);
    let socket = Arc::new(TokioUdpSocket::bind(&listener_addr).await.unwrap());
    let _ = election::bully_algorithm(port, curr_id, other_hosts,socket.clone()).await;
}


use std::io::{self, Write};
use tokio::task;
use std::net::{SocketAddr, UdpSocket};

pub mod config;
pub mod socket;
pub mod com;

use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3).await;

    let socket_server_1 = socket.socket_server_1;
    let socket_server_2 = socket.socket_server_2;
    let socket_server_3 = socket.socket_server_3;

    let message = "START";

    // Verify destination addresses before sending
    println!("Sending message to server 1: {}:{}", config.server_ip_1, config.port_send);
    println!("Sending message to server 2: {}:{}", config.server_ip_2, config.port_send);
    println!("Sending message to server 3: {}:{}", config.server_ip_3, config.port_send);

    // Task to send message to server 1
    let send_to_server_1 = {
        let socket_server_1_clone = socket_server_1.clone();
        tokio::spawn(async move {
            let dest = (config.server_ip_1, config.port_send);
            if let Err(e) = com::send(&socket_server_1_clone, message.to_string(), dest).await {
                eprintln!("Failed to send message to server 1: {}", e);
            }
        })
    };

    // Task to send message to server 2
    let send_to_server_2 = {
        let socket_server_2_clone = socket_server_2.clone();
        tokio::spawn(async move {
            let dest = (config.server_ip_2, config.port_send);
            if let Err(e) = com::send(&socket_server_2_clone, message.to_string(), dest).await {
                eprintln!("Failed to send message to server 2: {}", e);
            }
        })
    };

    // Task to send message to server 3
    let send_to_server_3 = {
        let socket_server_3_clone = socket_server_3.clone();
        tokio::spawn(async move {
            let dest = (config.server_ip_3, config.port_send);
            if let Err(e) = com::send(&socket_server_3_clone, message.to_string(), dest).await {
                eprintln!("Failed to send message to server 3: {}", e);
            }
        })
    };

    // Wait for all tasks to complete
    let _ = tokio::try_join!(send_to_server_3, send_to_server_2, send_to_server_1)?;

    Ok(())
}

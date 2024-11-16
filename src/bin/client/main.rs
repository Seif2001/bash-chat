use std::io::{self, Write};
use tokio::task;
use std::net::{SocketAddr, UdpSocket};

pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;

use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_rx).await;
    let config = Config::new();
    middleware::send_cloud(&socket, &config).await?;
    middleware::recv_leader(&socket).await?;

    Ok(())
}

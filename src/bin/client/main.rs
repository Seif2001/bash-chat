use std::io::{self, Write};
use tokio::task;
use std::net::{SocketAddr, UdpSocket};

pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;
pub mod image_com;

use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_leader_rx, config.address_client_tx, config.address_client_server_tx,config.address_client_dos_tx,config.address_client_dos_rx).await;
    let config = Config::new();
    middleware::send_cloud(&socket, &config,&"START".to_string()).await?;
    middleware::recv_leader(socket, config).await?;
    loop{
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    Ok(())
}





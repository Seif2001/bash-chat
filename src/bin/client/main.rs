use std::io::{self, Write};
use tokio::task;
use std::net::{SocketAddr, UdpSocket, Ipv4Addr};
use std::sync::Arc;

pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;
pub mod image_com;
pub mod dos;
pub mod api;
mod image_processor;


use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
#[show_image::main]

async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_leader_rx, config.address_client_tx, config.address_client_rx,config.address_client_dos_tx,config.address_client_dos_rx).await;
    // let socket = Arc::new(socket);
    // let config_arc = Arc::new(config); 
    
    // api::image_com_server(socket, config_arc).await.expect("Failed to start image com server");
    image_com::send_images_from_to(&socket, &config).await.expect("Failed to send images");

    Ok(())
}





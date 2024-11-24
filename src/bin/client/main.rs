use std::io::{self, Write};
use tokio::task;
use std::net::{SocketAddr, UdpSocket,Ipv4Addr};

pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;
pub mod image_com;
mod image_processor;


use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_leader_rx, config.address_client_tx, config.address_client_rx, config.address_client_server_tx,config.address_client_dos_tx,config.address_client_dos_rx).await;
    let config = Config::new();
    //middleware::send_cloud(&socket, &config,&"START".to_string()).await?;
    middleware::register_dos(&socket, &config).await?;
    let leader_ip = middleware::recv_leader(socket, config).await?;
    loop{
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // //Client 1 Config
    // let client_2_ip: Ipv4Addr = "10.7.16.154".parse().expect("Invalid IP address");
    // let client_2_address = (client_2_ip, 6384);
    // // Send "image Request"
    // middleware::p2p_send_image_request(&socket, client_2_address).await?;
    // // Send "Image Name"
    // middleware::p2p_send_image_name(&socket, client_2_address).await?;

    //Client 2 Config
    // Respond to "image Request"
    //middleware::p2p_recv_image_request(&socket).await?;
    // Respond to "Image Name"
    //middleware::p2p_recv_image_name(&socket).await?;

    Ok(())
}





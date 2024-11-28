use std::io::{self, Write};
use mini_redis::client;
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
mod image_store;


use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
#[show_image::main]

async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_leader_rx, config.address_client_tx, config.address_client_rx,config.address_client_dos_tx,config.address_client_dos_rx, config.address_client_image_request_rx).await;
    let config = Config::new();
    // //middleware::send_cloud(&socket, &config,&"START".to_string()).await?;
    // dos::register_dos(&socket, &config).await?;
    // dos::request_dos(&socket, &config).await?;
    // let mut clients = dos::parse_clients("clients_request.json",&config.username);
    // dos::print_clients(clients);
    // let leader_ip:Ipv4Addr= middleware::recv_leader(&socket, &config).await;
    // println!("Before send images");
    // image_com::send_images_from_to(&config.client_raw_images_dir, 1, 1, leader_ip, config.port_client_rx, &socket, &config).await?;
    // println!("After send images");
    image_store::create_json_for_images(&config.client_raw_images_dir, "my_images.json").unwrap();
    let client_ip = Ipv4Addr::new(10, 7, 16, 43);
    api::request_list_images(&socket, &config, client_ip).await?;
    Ok(())
}





use std::io::{self, Write};
use tokio::task;
use std::net::{SocketAddr, UdpSocket,Ipv4Addr};

pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;
pub mod image_com;
pub mod dos;

mod image_processor;


use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
#[show_image::main]

async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_leader_rx, config.address_client_tx, config.address_client_rx, config.address_client_server_tx,config.address_client_dos_tx,config.address_client_dos_rx).await;
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
    // image_processor::create_small_image("test.jpg".to_string(),"test_small.jpg".to_string());
    // image_processor::display_image("test_small.jpg");
    let client_ip: Ipv4Addr= Ipv4Addr::new(10,7,16,154);
    image_com::send_image(&socket, "image3.png", client_ip, config.port_client_rx, 1024, &config).await?;
    

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





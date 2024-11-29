use std::io::{self, Write};
use mini_redis::client;
use tokio::task;
use std::net::{SocketAddr, UdpSocket, Ipv4Addr};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;
pub mod image_com;
pub mod dos;
pub mod api;
mod image_processor;
mod image_store;
pub mod frontend;
use std::path::Path;

use crate::config::Config;
use crate::socket::Socket;

#[tokio::main]
#[show_image::main]

async fn main() -> io::Result<()> {
    let config = Config::new();
    let socket = Socket::new(
        config.address_server_1.clone(),
        config.address_server_2.clone(),
        config.address_server_3.clone(),
        config.address_client_leader_rx.clone(),
        config.address_client_tx.clone(),
        config.address_client_rx.clone(),
        config.address_client_dos_tx.clone(),
        config.address_client_dos_rx.clone(),
        config.address_client_image_request_rx.clone()
    ).await;
    // // frontend::start_frontend(&socket, &config).await?;
    // // //middleware::send_cloud(&socket, &config,&"START".to_string()).await?;
    // // dos::register_dos(&socket, &config).await?;
    // // dos::request_dos(&socket, &config).await?;
    // // let mut clients = dos::parse_clients("clients_request.json",&config.username);
    // // dos::print_clients(clients);
    // // let leader_ip:Ipv4Addr= middleware::recv_leader(&socket, &config).await;
    // // println!("Before send images");
    // // image_com::send_images_from_to(&config.client_raw_images_dir, 1, 1, leader_ip, config.port_client_rx, &socket, &config).await?;
    // // println!("After send images");
    // // image_store::create_json_for_images(&config.client_raw_images_dir, "my_images.json").unwrap();
    // let client_ip = Ipv4Addr::new(10, 7, 17, 170);
    // // api::request_list_images(&socket, &config, client_ip).await?;

    // // Client 2 Config
    // // Respond to "image Request"
    // // middleware::p2p_recv_image_request(&socket, &config).await?;
    // let sending_socket = socket.new_client_socket().await;
    // let image_name = "image3.png";
    // // let client_ip: Ipv4Addr = Ipv4Addr::new(10, 7, 19, 101);
    // let client_port = config.port_client_image_request_rx;
    // let _ = api::request_image(&socket, &config, sending_socket, image_name.to_string(), client_ip, client_port, true).await;
    // // // Respond to "Image Name"
    // // middleware::p2p_recv_request(&socket, &config).await?;
    // // let image_name = "image3.png";
    // // let image_path = Path::new(&config.client_encoded_images_dir).join(&image_name);
    // // println!("{:?}", image_processor::get_views(image_path.to_str().unwrap().to_string()));
    //  // Spawn a background task to handle incoming requests
    // Start the `p2p_recv_request` in a background task
    let config = Arc::new(config);
    let socket = Arc::new(Mutex::new(socket));

     tokio::spawn(async move {
        loop {
            // Lock the socket to get access to the inner Socket
            let socket_locked = socket.lock().await;
            if let Err(e) = middleware::p2p_recv_request(&*socket_locked, &config).await {
                eprintln!("Error in p2p_recv_request: {}", e);
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    println!("Received Ctrl+C, shutting down...");

    Ok(())
}







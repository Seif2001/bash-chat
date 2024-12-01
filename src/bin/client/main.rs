use std::io::{self, Write};
use async_std::path::Path;
use base64::read;
use mini_redis::client;
use time::convert::Nanosecond;
use tokio::sync::Mutex;
use tokio::task;
use std::net::{SocketAddr, UdpSocket, Ipv4Addr};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use std::{fs::File};
use std::{io::{Read}};
pub mod config;
pub mod socket;
pub mod com;
pub mod middleware;
pub mod image_com;
pub mod history_table;
pub mod dos;
pub mod api;
mod image_processor;
mod image_store;
pub mod frontend;

use crate::config::Config;
use crate::socket::{Socket};

pub async fn new_client_socket() -> Arc<Mutex<UdpSocket>>{
    // bind random available port
    let socket = Arc::new(Mutex::new(UdpSocket::bind("0.0.0.0:0").expect("Error binding")));
    socket
}

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
    
    // // Wrap the socket and config in Arc<Mutex<>> to share across tasks
    let config_arc = Arc::new(config);
    let socket_arc = Arc::new(socket);
    // // image_com::send_images_from_to(&config.client_raw_images_dir, 1, 1, Ipv4Addr::new(10, 7, 16, 43), config.port_client_rx, &socket_arc, &config).await?;
    dos::register_dos(&socket_arc, &config_arc).await?;
    dos::request_dos(&socket_arc, &config_arc).await?;
    let socket_arc_clone = Arc::clone(&socket_arc);
    let config_clone = Arc::clone(&config_arc);
    let json_path = "image_requests_unfinished.json";
    let requests = match image_processor::read_image_requests(json_path) {
        Ok(requests) => requests,
        Err(e) => {
            eprintln!("Failed to read or parse the JSON file: {}", e);
            return Err(e);
        }
    };
    for request in requests {
        // Assuming `request_image` is the function to send a request
        println!("Making the request for image: {}", request.image_name);
        let sending_socket = socket_arc.new_client_socket().await;
        let client_ip: Ipv4Addr = dos::get_ip_by_username_as_ipv4(&request.client_username)?;
        let _ = api::request_image(&socket_arc, &config_clone, sending_socket, request.image_name, client_ip, config_clone.port_client_image_request_rx, request.is_high).await;
    }
    let socket_arc_clone = Arc::clone(&socket_arc);
    let config_clone = Arc::clone(&config_arc);
    let _ = tokio::spawn({
        async move {
            let _ = middleware::p2p_recv_request(&socket_arc_clone, &config_clone).await;
        }
    });

    
    let config = Config::new();
    let socket = socket_arc.clone();
    frontend::run(&*socket, config).await;
    // // Respond to "Image Name"
    //let _ =api::receive_image_request(&socket, &config).await;
        
        Ok(())
    }
    
    

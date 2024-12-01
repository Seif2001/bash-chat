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
    
    // // Wrap the socket and config in Arc<Mutex<>> to share across tasks
    let config_arc = Arc::new(config);
    let socket_arc = Arc::new(socket);
    // // image_com::send_images_from_to(&config.client_raw_images_dir, 1, 1, Ipv4Addr::new(10, 7, 16, 43), config.port_client_rx, &socket_arc, &config).await?;
    dos::register_dos(&socket_arc, &config_arc).await?;
    // dos::request_dos(&socket_arc, &config).await?;

    let socket_arc_clone = Arc::clone(&socket_arc);
    let config_clone = Arc::clone(&config_arc);
    let _ = tokio::spawn({
        async move {
            let _ = middleware::p2p_recv_request(&socket_arc_clone, &config_clone).await;
        }
    });
    // dos::request_dos(&socket_arc, &config).await?;
    // // let leader_ip: Ipv4Addr = middleware::recv_leader(&socket_arc, &config).await;
    // // println!("Leader is {} ", leader_ip);
    // // println!("Before send images");
    // // image_com::send_images_to_server(&config.client_raw_images_dir, 1, 1, leader_ip, config.port_client_rx, &socket_arc, &config).await?;
    // // println!("After send images");
    // loop{}
    //         // middleware::send_cloud(&socket, &config,&"START".to_string()).await?;

    //     // let mut clients = dos::parse_clients("clients_request.json",&config.username);
    //     // dos::print_clients(clients);
    //     // image_store::create_json_for_images(&config.client_raw_images_dir, "my_images.json").unwrap();
    //     // let client_ip = Ipv4Addr::new(10, 7, 16, 43);
    //     // api::request_list_images(&socket, &config, client_ip).await?;
    
    //     // Client 2 Config
    //     // Respond to "image Request"
    //     // middleware::p2p_recv_image_request(&socket, &config).await?;
    //      let sending_socket = socket.new_client_socket().await;
    //      let image_name = "image3.png";
    //     // let client_ip: Ipv4Addr = Ipv4Addr::new(10, 7, 19, 101);
    //     let client_ip: Ipv4Addr = dos::get_ip_by_username_as_ipv4(&"yehia")?;
    //     let client_port = config.port_client_image_request_rx;
    //     let _ = api::request_image(&socket, &config, sending_socket, image_name.to_string(), client_ip, client_port, false).await;
    //     let high_path = Path::new(&config.client_high_quality_receive_dir).join(&image_name);
    //     image_processor::display_image(&high_path.display().to_string());
    let config = Config::new();
    let socket = socket_arc.clone();
    frontend::run(&*socket, config).await;
    // // Respond to "Image Name"
    //let _ =api::receive_image_request(&socket, &config).await;
        
        Ok(())
    }
    
    

use std::io::{self, Write};
use tokio::sync::mpsc;
use std::net::{SocketAddr, Ipv4Addr};
use std::sync::Arc;

use crate::config::Config;
use crate::socket::Socket;
use crate::{image_com, middleware};

pub async fn image_com_server(socket: Arc<Socket>, config: Arc<Config>) -> io::Result<()> {
    let start = "START".to_string();
    let socket_clone = Arc::clone(&socket);
    let config_clone = Arc::clone(&config);

    let socket_client = socket_clone.new_client_socket().await;

    // Create a channel to send the leader IP back
    let (tx, mut rx) = mpsc::channel(1); // A channel with a buffer size of 1

    middleware::send_cloud(&socket, &config, &start).await.expect("Failed to send to cloud");

    tokio::spawn({
        async move {
            let leader_ip = middleware::recv_leader(&socket_clone, &config_clone).await;

            // Send the leader IP back to the main scope using the channel
            if let Err(e) = tx.send(leader_ip).await {
                eprintln!("Failed to send leader IP: {}", e);
            }
        }
    });

    // Await the leader IP from the channel
    let leader_ip = rx.recv().await.expect("Failed to receive leader IP");
    
    // Now you can use the leader_ip outside the task
    println!("Received leader IP: {}", leader_ip);

    Ok(())
}


// pub async fn request_image(socket: &Socket, config: &Config, image_name: String, client_ip: Ipv4Addr) -> io::Result<()>{
//     let request_message = "GET " + image_name;
//     middleware::p2p_send_image_request(socket, config, client_address, request_message);
//     image_com::receive_image(socket, config);
// }

pub async fn request_list_images(socket: &Socket, config: &Config, client_ip: Ipv4Addr) -> io::Result<()>{
    let request_message = "GET LIST";
    middleware::p2p_send_image_request(socket, config, client_ip, request_message).await;
    Ok(())
}



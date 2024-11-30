use std::io::{self, Write};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
use std::net::{SocketAddr, Ipv4Addr};
use std::sync::Arc;

use crate::config::Config;
use crate::socket::{self, Socket};
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

pub async fn request_list_images(socket: &Socket, config: &Config, client_ip: Ipv4Addr) -> io::Result<()> {
    let request_message = "GET LIST".to_string();
    let socket_tx_rx = socket.new_client_socket().await;
    let socket_tx_rx_clone = Arc::clone(&socket_tx_rx);
    // Try to send the image list request and handle errors
    match middleware::p2p_send_list_images_request(socket, config, client_ip, &request_message, socket_tx_rx).await {
        Ok(_) => {
            //println!("Image list request sent successfully. Now waiting for the list of images.");

            // If sending was successful, try to receive the image list
            if let Err(e) = middleware::p2p_recv_list_images(socket_tx_rx_clone).await {
                eprintln!("Error receiving list of images: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Error sending image list request: {}", e);
        }
    }

    Ok(())
}


pub async fn request_image(
    socket: &Socket,
    config: &Config,
    sending_socket: Arc<Mutex<UdpSocket>>,
    image_name: String,
    client_ip: Ipv4Addr,
    client_port: u16,
    is_high: bool
) -> io::Result<()> {
    // Determine the request message based on the quality flag
    let request_message = if is_high {
        format!("GET H {}", image_name)
    } else {
        format!("GET L {}", image_name)
    };

    // Determine the correct path for saving the image
    let received_path = if is_high {
        async_std::path::PathBuf::from(&config.client_high_quality_receive_dir)
    } else {
        async_std::path::PathBuf::from(&config.client_low_quality_receive_dir)
    };

    // Attempt to send the image request
    match middleware::p2p_send_image_request(socket, sending_socket.clone(), config, client_ip, client_port, &request_message, received_path.clone()).await {
        Ok(_) => {
            // If the request is successful, proceed to receiving and saving the image
            image_com::receive_image(socket, config, sending_socket, received_path).await?;
            Ok(())
        }
        Err(e) => {
            // If there is an error in sending the request, handle the error
            eprintln!("Failed to send image request: {}", e);
            Err(e)
        }
    }
}



pub async fn receive_image_request(
    socket: &Socket,
    config: &Config,
) {
    match middleware::p2p_recv_request(socket, config).await {
        Ok(_) => {
        }
        Err(e) => {
            // Log the error but don't stop the program
            eprintln!("error: {}", e);
        }
    }
}
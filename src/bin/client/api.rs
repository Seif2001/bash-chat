use std::io::{self, Write};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
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


pub async fn request_image(
    socket: &Socket,
    config: &Config,
    sending_socket: Arc<Mutex<UdpSocket>>,
    image_name: String,
    client_ip: Ipv4Addr,
    client_port: u16,
    is_high: bool
) -> io::Result<()> {
    let request_message = if is_high {
        format!("GET H {}", image_name)
    } else {
        format!("GET L {}", image_name)
    };

    // Attempt to send the image request
    //low quality
    let low_path = async_std::path::PathBuf::from(config.client_low_quality_images_dir.clone());
    match middleware::p2p_send_image_request(socket, sending_socket.clone(), config, client_ip,client_port, &request_message, low_path.clone()).await {
        Ok(_) => {
            // If the request is successful, proceed to sending the image
            image_com::receive_image(socket, config, sending_socket, low_path).await?;
            Ok(())
        }
        Err(e) => {
            // If there is an error in sending the request, handle the error
            eprintln!("Failed to send image request: {}", e);
            // Optionally, you can try to recover, retry, or just return the error
            Err(e)
        }
    }
}



use mini_redis::server;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;
use crate::image_com;
use std::fs::{File, read_dir, create_dir_all};
use std::path::Path;
use std::net::Ipv4Addr;
use std::io::Write;
use std::io::Read;

async fn send_images_from_to(
    image_path: &str,
    mut num_images: usize,
    client_order: u32,
    server_ip: Ipv4Addr,
    server_port: u16,
    send_socket: &Socket,
    config: &Config
) -> std::io::Result<()> {

    println!("\n******************************************************************************");
    println!(
        "Client {} is sending {} images from '{}' to addr {}:{}",
        client_order, 
        num_images,
        image_path, 
        server_ip,
        server_port
    );
    println!("********************************************************************************");


    for entry in read_dir(image_path)? {
        if num_images == 0 {
            break;
        }
        num_images -= 1;

        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                
                println!(" \n >>>>>>>>>>>>>>>> file: {} <<<<<<<<<<<<<<<", file_name);
                
                // Send the "START" message directly to the server
                let dest = (server_ip, server_port);
                com::send(&send_socket.socket_client_server_tx, "START".to_string(), dest).await?;
                println!(" --  'START' message sent ");

                // Receive leader's address from the server
                let (ack, _) = com::recv(&send_socket.socket_client_rx).await?;
                println!(" --  received ack to begin sending: {}", ack); // already inside the receive leader function

                image_com::send_image(&send_socket, async_std::path::Path::new(image_path), server_ip, server_port, 1024).await?;


                // Wait for acknowledgment from the server
                image_com::receive_image(&send_socket, config).await?;
            }
        }
    }

    println!("\nClient {} completed sending images to addr {}.", client_order, server_ip);
    println!("----------------------------------------------------------------------------");
    println!("----------------------------------------------------------------------------\n");

    Ok(())
}

// send to 3 servers

pub async fn send_cloud(socket: &Socket, config: &Config, message: &String)  -> std::io::Result<()>  {
    let socket_server_1 = &socket.socket_server_1;
    let socket_server_2 = &socket.socket_server_2;
    let socket_server_3 = &socket.socket_server_3;


    // Verify destination addresses before sending
    println!("Sending message to server 1: {}:{}", config.server_ip_1, config.port_client_elections_rx);
    println!("Sending message to server 2: {}:{}", config.server_ip_2, config.port_client_elections_rx);
    println!("Sending message to server 3: {}:{}", config.server_ip_3, config.port_client_elections_rx);

    // Task to send message to server 1
    let message_copy = message.clone();
    let message_copy_2 = message.clone();
    let message_copy_3 = message.clone();


    let send_to_server_1 = {
        let socket_server_1_clone = socket_server_1.clone();
        let server_ip_1 = config.server_ip_1.clone();
        let port_send_1 = config.port_client_elections_rx;
        tokio::spawn(async move {
            let dest = (server_ip_1, port_send_1);
            if let Err(e) = com::send(&socket_server_1_clone, message_copy, dest).await {
                eprintln!("Failed to send message to server 1: {}", e);
            }
        })
    };

    // Task to send message to server 2
    let send_to_server_2 = {
        let socket_server_2_clone = socket_server_2.clone();
        let server_ip_2 = config.server_ip_2.clone();
        let port_send_2 = config.port_client_elections_rx;
        tokio::spawn(async move {
            let dest = (server_ip_2, port_send_2);
            if let Err(e) = com::send(&socket_server_2_clone, message_copy_2, dest).await {
                eprintln!("Failed to send message to server 2: {}", e);
            }
        })
    };

   
    let send_to_server_3 = {
        let socket_server_3_clone = socket_server_3.clone();
        let server_ip_3 = config.server_ip_3.clone();
        let port_send_3 = config.port_client_elections_rx;
        tokio::spawn(async move {
            let dest = (server_ip_3, port_send_3);
            if let Err(e) = com::send(&socket_server_3_clone, message_copy_3, dest).await {
                eprintln!("Failed to send message to server 3: {}", e);
            }
        })
    };

    // Wait for all tasks to complete
    let _ = tokio::try_join!(send_to_server_3, send_to_server_2, send_to_server_1)?;
    Ok(())
}


pub async fn send_cloud_port(socket: &Socket, config: &Config, message: &String, port: u16) -> std::io::Result<()> {
    let socket_server_1 = &socket.socket_server_1;
    let socket_server_2 = &socket.socket_server_2;
    let socket_server_3 = &socket.socket_server_3;

    // Verify destination addresses before sending
    println!("Sending message to server 1: {}:{}", config.server_ip_1, port);
    println!("Sending message to server 2: {}:{}", config.server_ip_2, port);
    println!("Sending message to server 3: {}:{}", config.server_ip_3, port);

    // Task to send message to server 1
    let message_copy = message.clone();
    let message_copy_2 = message.clone();
    let message_copy_3 = message.clone();

    let send_to_server_1 = {
        let socket_server_1_clone = socket_server_1.clone();
        let server_ip_1 = config.server_ip_1.clone();
        tokio::spawn(async move {
            let dest = (server_ip_1, port);
            if let Err(e) = com::send(&socket_server_1_clone, message_copy, dest).await {
                eprintln!("Failed to send message to server 1: {}", e);
            }
        })
    };

    // Task to send message to server 2
    let send_to_server_2 = {
        let socket_server_2_clone = socket_server_2.clone();
        let server_ip_2 = config.server_ip_2.clone();
        tokio::spawn(async move {
            let dest = (server_ip_2, port);
            if let Err(e) = com::send(&socket_server_2_clone, message_copy_2, dest).await {
                eprintln!("Failed to send message to server 2: {}", e);
            }
        })
    };

    // Task to send message to server 3
    let send_to_server_3 = {
        let socket_server_3_clone = socket_server_3.clone();
        let server_ip_3 = config.server_ip_3.clone();
        tokio::spawn(async move {
            let dest = (server_ip_3, port);
            if let Err(e) = com::send(&socket_server_3_clone, message_copy_3, dest).await {
                eprintln!("Failed to send message to server 3: {}", e);
            }
        })
    };

    // Wait for all tasks to complete
    let _ = tokio::try_join!(send_to_server_1, send_to_server_2, send_to_server_3)?;

    Ok(())
}

// receive leader message

pub async fn recv_leader(socket: Socket, config: Config) -> std::io::Result<()> {
    let socket_client_leader_rx = socket.socket_client_leader_rx.clone();
    let socket_client_tx = socket.socket_client_tx.clone();

    tokio::spawn({
        async move {
            loop {
            let (message, src) = com::recv(&socket_client_leader_rx).await.expect("Failed to receive message");
            let message = message.trim();
            let src = src.ip();
            
                if message.starts_with("LEADER") {
                    println!("Received leader message from {}: {}", src, message);
                    if let std::net::IpAddr::V4(ipv4_src) = src {
                        return ipv4_src;
                    } else {
                        eprintln!("Received non-IPv4 address: {}", src);
                    }
                
                }
            }
        }
    });
    Ok(())
}

pub async fn register_dos(socket: &Socket, config: &Config) -> std::io::Result<()> {
    let socket_client_dos_rx = socket.socket_client_dos_rx.clone(); // port on client
    let register_message = format!("REGISTER {}", config.username);  // Register to dos message

    let (ack_tx, mut ack_rx): (broadcast::Sender<bool>, broadcast::Receiver<bool>) = broadcast::channel(1); // tx `true` when ack is received

    // Task that listens for an ACK
    tokio::spawn({
        let socket_client_dos_rx = socket_client_dos_rx.clone();
        let ack_tx = ack_tx.clone();
        async move {
            loop {
                // Receive a message from the server
                let (message, _src) = com::recv(&socket_client_dos_rx)
                    .await
                    .expect("Failed to receive message");
                let message = message.trim();

                if message == "ACK" {
                    println!("Received ACK from server.");
                    let _ = ack_tx.send(true); // Send signal that ACK was received
                    break; // Exit the loop once ACK is received
                }
            }
        }
    });

    // Loop to try registering every 5 seconds until we receive an ACK
    loop {
        send_cloud_port(&socket, &config, &register_message, config.port_client_dos_tx).await?;
        tokio::select! {
            _ = ack_rx.recv() => {
                println!("ACK received, registration successful.");
                break;
            }
            _ = sleep(Duration::from_secs(5)) => {
                println!("Timeout, sending REGISTER again...");
            }
        }
    }

    Ok(())
}

pub async fn read_file(message: String) -> std::io::Result<()> {
    let file_path = "clients_request.json";

    let mut file = File::create(file_path)?;
    file.write_all(message.as_bytes())?;

    Ok(())
}

pub async fn request_dos(socket: &Socket, config: &Config) -> std::io::Result<()> {
    let socket_client_dos_rx = socket.socket_client_dos_rx.clone(); // port on client
    let message = "REQUEST";

    let (ack_tx, mut ack_rx): (broadcast::Sender<bool>, broadcast::Receiver<bool>) = broadcast::channel(1); // tx `true` when ack is received

    // Task that listens for an json file
    tokio::spawn({
        let socket_client_dos_rx = socket_client_dos_rx.clone();
        let ack_tx = ack_tx.clone();
        async move {
            loop {
                let (message, _src) = com::recv(&socket_client_dos_rx)
                    .await
                    .expect("Failed to receive message");
                if let Err(e) = read_file(message.to_string()).await {
                    eprintln!("Failed to process received file: {}", e);
                }
            }
        }
    });

    // Loop to try registering every 5 seconds until we receive an ACK
    loop {
        send_cloud_port(&socket, &config, &message.to_string(), config.port_client_dos_tx).await?;
        tokio::select! {
            _ = ack_rx.recv() => {
                println!("ACK received, registration successful.");
                break;
            }
            _ = sleep(Duration::from_secs(5)) => {
                println!("Timeout, sending REGISTER again...");
            }
        }
    }

    Ok(())
}

//P2P Communication
// Function for sending an "image Request" to Client 2
pub async fn p2p_send_image_request(socket: &Socket, client_address: (std::net::Ipv4Addr, u16)) -> std::io::Result<()> {
    let socket_client_tx = &socket.socket_client_tx;
    let message = "image Request";

    // Broadcast channel to signal when acknowledgment is received
    let (ack_tx, mut ack_rx) = tokio::sync::broadcast::channel(1);

    // Task to listen for acknowledgment
    tokio::spawn({
        let socket_client_tx = socket_client_tx.clone();
        let ack_tx = ack_tx.clone();
        async move {
            loop {
                let (response, src) = com::recv(&socket_client_tx).await.expect("Failed to receive message");
                let response = response.trim();

                if response == "Sample Images" {
                    println!("Received acknowledgment 'Sample Images' from {}", src);
                    let _ = ack_tx.send(true); // Signal acknowledgment received
                    break;
                } else {
                    println!("Received unexpected response '{}'", response);
                }
            }
        }
    });

    // Retry loop to send the request
    loop {
        println!("Sending 'image Request' to {}:{}", client_address.0, client_address.1);
        com::send(socket_client_tx, message.to_string(), client_address).await?;

        tokio::select! {
            _ = ack_rx.recv() => {
                println!("Acknowledgment received, moving to next step.");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                println!("Timeout, resending 'image Request'...");
            }
        }
    }

    Ok(())
}


pub async fn p2p_recv_image_request(socket: &Socket) -> std::io::Result<()> {
    let client_rx = &socket.socket_client_rx;
    let socket_client_tx = &socket.socket_client_tx;

    loop {
        let (message, src) = com::recv(client_rx).await?;
        let message = message.trim();

        if message == "image Request" {
            println!("Received 'image Request' from {}", src);

            let response = "Sample Images";
            println!("Responding with 'Sample Images' to {}", src);
            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                com::send(socket_client_tx, response.to_string(), (ipv4_src, src.port())).await?;
            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
            break; // Exit loop after responding
        } else {
            println!("Received unexpected message '{}'", message);
        }
    }

    Ok(())
}


// Function for sending an image name from Client 1 to Client 2
pub async fn p2p_send_image_name(socket: &Socket, client_address: (std::net::Ipv4Addr, u16)) -> std::io::Result<()> {
    let socket_client_tx = &socket.socket_client_tx;
    let image_name = "Image Name";

    // Broadcast channel to signal when acknowledgment is received
    let (ack_tx, mut ack_rx) = tokio::sync::broadcast::channel(1);

    // Task to listen for acknowledgment
    tokio::spawn({
        let socket_client_tx = socket_client_tx.clone();
        let ack_tx = ack_tx.clone();
        async move {
            loop {
                let (response, src) = com::recv(&socket_client_tx).await.expect("Failed to receive message");
                let response = response.trim();

                if response == image_name {
                    println!("Received acknowledgment '{}' from {}", response, src);
                    let _ = ack_tx.send(true); // Signal acknowledgment received
                    break;
                } else {
                    println!("Received unexpected response '{}'", response);
                }
            }
        }
    });

    // Retry loop to send the image name
    loop {
        println!("Sending image name '{}' to {}:{}", image_name, client_address.0, client_address.1);
        com::send(socket_client_tx, image_name.to_string(), client_address).await?;

        tokio::select! {
            _ = ack_rx.recv() => {
                println!("Acknowledgment received, image name sent successfully.");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                println!("Timeout, resending image name...");
            }
        }
    }

    Ok(())
}


// Function for receiving an image name and responding with confirmation
pub async fn p2p_recv_image_name(socket: &Socket) -> std::io::Result<()> {
    let socket_client_rx = &socket.socket_client_rx;

    loop {
        let (message, src) = com::recv(socket_client_rx).await?;
        let message = message.trim();

        if message == "Image Name" {
            println!("Received image name '{}' from {}", message, src);
            println!("Echoing back image name '{}' to {}", message, src);
            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                com::send(socket_client_rx, message.to_string(), (ipv4_src, src.port())).await?;
            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
            break; // Exit loop after responding
        } else {
            println!("Received unexpected message '{}'", message);
        }
    }

    Ok(())
}

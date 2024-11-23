
use mini_redis::server;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;
use crate::image_com;

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
    

    tokio::spawn({
        async move {
            loop {
            let (message, src) = com::recv(&socket_client_leader_rx).await.expect("Failed to receive message");
            let message = message.trim();
            let src = src.ip();
            
                if message.starts_with("LEADER") {
                    println!("Received leader message from {}: {}", src, message);
                    if let std::net::IpAddr::V4(ipv4_src) = src {
                        image_com::send_image(ipv4_src, &socket, &config).await.expect("Failed to send image");
                    } else {
                        eprintln!("Received non-IPv4 address: {}", src);
                    }
                
                }
            }
        }
    });
    Ok(())
}

pub async fn register_dos(socket: Socket, config: Config) -> std::io::Result<()> {
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
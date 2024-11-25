
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
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Client {
    id: u32,
    ip: String,
    username: String,
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
                else{
                    println!("File received successfully.");
                    let _ = ack_tx.send(true);
                    break; 
                }
            }
        }
    });

    // Loop to try request dos every 5 seconds until we receive an ACK
    loop {
        send_cloud_port(&socket, &config, &message.to_string(), config.port_client_dos_tx).await?;
        tokio::select! {
            _ = ack_rx.recv() => {
                println!("file received, request successful.");
                break;
            }
            _ = sleep(Duration::from_secs(5)) => {
                println!("Timeout, sending REQUEST again...");
            }
        }
    }

    Ok(())
}
pub fn parse_clients(json_path: &str, curr_client: &str) -> Vec<Client>{
    let json_data = fs::read_to_string(json_path)
        .expect("Failed to read JSON file from the provided path");

    let all_clients: Vec<Client> = serde_json::from_str(&json_data)
        .expect("Failed to parse JSON data");

    let filtered_clients: Vec<Client> = all_clients
        .into_iter()
        .filter(|user| user.username != curr_client)
        .collect();

    filtered_clients
}


pub fn print_clients(clients: Vec<Client>) {
    for client in clients {
        println!("Client {}. {}", client.id, client.username);
    }
}
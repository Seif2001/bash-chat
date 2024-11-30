
use async_std::path::PathBuf;
use mini_redis::server;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, Mutex};
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};
use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::{com, dos, image_processor};
use crate::image_com;
use std::fs::{File, read_dir, create_dir_all};
use std::path::Path;
use std::net::Ipv4Addr;
use std::io::Write;
use std::io::Read;
use serde_json::Value; // For deserializing the received JSON (if needed)
use serde::{Serialize, Deserialize};
use crate::history_table;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use tokio::fs::read_to_string;

// Define the Image struct and ImageList type (a vector of Image)
#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub image: String,
    pub views: u32,
}

pub type ImageList = Vec<Image>;
async fn get_views_for_image(image_name: &str, json_path: &str) -> Result<u32, std::io::Error> {
    // Read the JSON file asynchronously
    let file_contents = read_to_string(json_path).await?;
    
    // Parse the JSON into a vector of ImageEntry objects
    let image_entries: Vec<Image> = serde_json::from_str(&file_contents)?;

    // Search for the image entry by image name and return the views
    for entry in image_entries {
        if entry.image == image_name {
            return Ok(entry.views);
        }
    }
    
    // If image is not found, return a default value (e.g., 0 views)
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Image not found"))
}

// async fn send_images_from_to(
//     image_path: &str,
//     mut num_images: usize,
//     client_order: u32,
//     server_ip: Ipv4Addr,
//     server_port: u16,
//     send_socket: &Socket,
//     config: &Config
// ) -> std::io::Result<()> {

//     println!("\n******************************************************************************");
//     println!(
//         "Client {} is sending {} images from '{}' to addr {}:{}",
//         client_order, 
//         num_images,
//         image_path, 
//         server_ip,
//         server_port
//     );
//     println!("********************************************************************************");


//     for entry in read_dir(image_path)? {
//         if num_images == 0 {
//             break;
//         }
//         num_images -= 1;

//         let entry = entry?;
//         let path = entry.path();

//         if path.is_file() {
//             if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                
//                 println!(" \n >>>>>>>>>>>>>>>> file: {} <<<<<<<<<<<<<<<", file_name);
                
//                 // Send the "START" message directly to the server
//                 let dest = (server_ip, server_port);
//                 com::send(&send_socket.socket_client_server_tx, "START".to_string(), dest).await?;
//                 println!(" --  'START' message sent ");

//                 // Receive leader's address from the server
//                 let (ack, _) = com::recv(&send_socket.socket_client_rx).await?;
//                 println!(" --  received ack to begin sending: {}", ack); // already inside the receive leader function

//                 image_com::send_image(&send_socket, async_std::path::Path::new(image_path), server_ip, server_port, 1024).await?;


//                 // Wait for acknowledgment from the server
//                 image_com::receive_image(&send_socket, config).await?;
//             }
//         }
//     }

//     println!("\nClient {} completed sending images to addr {}.", client_order, server_ip);
//     println!("----------------------------------------------------------------------------");
//     println!("----------------------------------------------------------------------------\n");

//     Ok(())
// }

// send to 3 servers

pub async fn send_cloud(socket: &Socket, config: &Config, message: &String)  -> std::io::Result<()>  {
    let socket_server_1 = &socket.new_client_socket().await;
    let socket_server_2 = &socket.new_client_socket().await;
    let socket_server_3 = &socket.new_client_socket().await;


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


pub async fn recv_leader(socket: &Socket, config: &Config) -> Ipv4Addr {
    let socket_client_leader_rx = socket.socket_client_leader_rx.clone();
    let socket_client_tx = socket.socket_client_tx.clone();

    send_cloud(&socket, &config, &"START".to_string()).await.expect("Failed to send message");
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

//P2P Communication
// Function for sending an "image Request" to Client 2use tokio::time::{timeout, Duration};

pub async fn p2p_send_image_request(
    socket: &Socket,
    sending_socket: Arc<Mutex<UdpSocket>>,
    config: &Config,
    client_address: Ipv4Addr,
    client_port: u16,
    message: &String,
    path: PathBuf,
) -> std::io::Result<()> {
    let (ack_tx, mut ack_rx) = tokio::sync::broadcast::channel(1);
    let max_retries = 5; // Number of retries
    let mut attempts = 0;
    let receive_timeout = Duration::from_secs(1); // Timeout duration for receiving

    // Task to listen for acknowledgment
    tokio::spawn({
        let sending_socket = sending_socket.clone();
        let ack_tx = ack_tx.clone();
        async move {
            let mut attempts_here =0;
            loop {
                // Apply timeout to the receive operation
                let receive_result = timeout(receive_timeout, com::recv(&sending_socket)).await;
                if attempts_here >= max_retries {
                    break;
                }
                else{
                    attempts_here+=1;
                }
                match receive_result {
                    Ok(Ok((response, _src))) => {
                        let response = response.trim();
                        if response == "ack_request" {
                            let _ = ack_tx.send(true); // Signal acknowledgment received
                            break;
                        } else {
                            println!("Received unexpected response: '{}'", response);
                        }
                    }
                    Ok(Err(e)) => {
                        println!("Error receiving message: {}", e);
                    }
                    Err(_) => {
                        println!("Timeout waiting for acknowledgment");
                    }
                }
            }
        }
    });

    while attempts < max_retries {
        attempts += 1;
        println!("Sending image request to {}:{}", client_address, client_port);
        let dest = (client_address, client_port);
        {
            com::send(&sending_socket, message.to_string(), dest).await?;
        }

        tokio::select! {
            _ = ack_rx.recv() => {
                println!("Acknowledgment received.");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                println!("Timeout waiting for acknowledgment, resending request... Attempt {}/{}", attempts, max_retries);
            }
        }
    }

    if attempts >= max_retries {
        return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "Max retries reached"));
    }

    Ok(())
}



pub async fn p2p_recv_request(socket: &Socket, config: &Config) -> std::io::Result<()> {
    let client_rx = &socket.socket_client_request_images_rx;

    loop {
        let (message, src) = com::recv(client_rx).await?;
        let message = message.trim();
        if message.starts_with("GET H ") {
            let image_name = message.trim_start_matches("GET H ").trim().to_string();
            println!("Received high image Request from {}", src);
            if image_name.is_empty() {
                println!("No image name provided");
                continue;
            }
            let response = "ack_request";
            let sending_socket = socket.new_client_socket().await;
            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                com::send(&sending_socket, response.to_string(), (ipv4_src, src.port())).await?;
                println!("Sent acknowledgment to {}", src);

                let path = config.client_encoded_images_dir.clone();
                //send_cloud(&socket, &config,&"START".to_string()).await?;
                //let leader_ip:Ipv4Addr= recv_leader(&socket, &config).await;
                //save to history table
                //_ = dos::request_dos(&socket, &config).await;
                //let requester_username = dos::get_username_by_ip(&ipv4_src.to_string())?;
                //let _ = history_table::add_to_history(&config.username,&requester_username,&image_name);
                //image_com::send_images_from_to(&config.client_raw_images_dir, &image_name, 1, leader_ip, config.port_client_rx, &socket, &config).await?;
                // println!("After send images");
                println!("after hist table");
                let temp_image_path = Path::new(&config.client_encoded_images_dir).join(&image_name);
                // Define the path to the JSON file
                let json_path = "my_images.json"; // Replace with the actual path to the JSON file
                
                // Retrieve the number of views for the image from the JSON file
                let views = match get_views_for_image(&image_name, json_path).await {
                    Ok(views) => views,
                    Err(_) => {
                        println!("Image not found in JSON, defaulting to 0 views.");
                        0 // Default to 0 views if image is not found
                    }
                };
                image_processor::append_views(temp_image_path.display().to_string(), temp_image_path.display().to_string(),views);
                println!("before send images");
                if let Ok(_) = image_com::send_image(socket, &image_name, &path, ipv4_src, src.port(), 1020, config).await {
                    println!("Image sent successfully!");
                    //let _ = history_table::mark_as_sent(&config.username,&requester_username,&image_name);
                } else {
                    println!("Failed to send image.");
                }
                //image_com::send_image(socket, &image_name, &path, ipv4_src, src.port(), 1020, config).await?;

            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
            // break; there is no breaking man we keep listeening forever and ever and ever
        } 
        else if message.starts_with("GET L ") {
            let image_name = message.trim_start_matches("GET L ").trim().to_string();

            println!("Received low image Request from {}", src);
            if image_name.is_empty() {
                println!("No image name provided");
                continue;
            }
            let response = "ack_request";
            let sending_socket = socket.new_client_socket().await;

            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                com::send(&sending_socket, response.to_string(), (ipv4_src, src.port())).await?;
                let image_path = Path::new(&config.client_raw_images_dir).join(&image_name);
                let low_image_path = Path::new(&config.client_low_quality_images_dir).join(&image_name);
                println!("Creating low image: {}", low_image_path.display());
                println!("input path: {}", image_path.display());
                image_processor::resize_image(image_path.to_str().unwrap(), low_image_path.to_str().unwrap()); //add low image directory
                let low_path = config.client_low_quality_images_dir.clone();
                image_com::send_image(socket, &image_name, &low_path, ipv4_src, src.port(), 1020, config).await?;
            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
            // break; there is no breaking man we keep listeening forever and ever and ever
        } 
        else if message == "GET LIST" {

            let response = "ack_list";

            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                let dest = (ipv4_src, src.port()); // This is the destination address and port

    
                tokio::spawn(async move {
                    
                    if let Err(e) = com::send(&socket::new_client_socket().await, response.to_string(), dest).await {
                        eprintln!("Error sending acknowledgment: {}", e);
                    } else {
                        println!("HERE");
                        if let Err(e) = p2p_send_images_list(&socket::new_client_socket().await, dest).await {
                            eprintln!("Error sending images list: {}", e);
                        }
                    }
                });
            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
        }
            
        
        else {
            println!("Received unexpected message '{}'", message);
        }
    }

    Ok(())
}

pub async fn p2p_recv_request_copy(socket: &Socket, config: &Config) -> std::io::Result<()> {
    let client_rx = &socket.socket_client_request_images_rx;

    loop {
        let (message, src) = com::recv(client_rx).await?;
        let message = message.trim();
        if message.starts_with("GET H ") {
            let image_name = message.trim_start_matches("GET H ").trim().to_string();
            println!("Received high image Request from {}", src);
            if image_name.is_empty() {
                println!("No image name provided");
                continue;
            }
            let response = "ack_request";
            let sending_socket = socket.new_client_socket().await;
            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                com::send(&sending_socket, response.to_string(), (ipv4_src, src.port())).await?;
                let leader_ip:Ipv4Addr= recv_leader(&socket, &config).await;
                let path = config.client_encoded_images_dir.clone();
                send_cloud(&socket, &config,&"START".to_string()).await?;
                let leader_ip:Ipv4Addr= recv_leader(&socket, &config).await;
                println!("Before send images");
                image_com::send_images_from_to(&config.client_raw_images_dir, &image_name, 1, leader_ip, config.port_client_rx, &socket, &config).await?;
                println!("After send images");
                // image_com::send_images_from_to(&config.client_raw_images_dir, 1, 1, leader_ip, config.port_client_rx, &socket, &config).await?;
                //save to history table
                //_ = dos::request_dos(&socket, &config).await;
                //let requester_username = dos::get_username_by_ip(&ipv4_src.to_string())?;
                //let _ = history_table::add_to_history(&config.username,&requester_username,&image_name);

                if let Ok(_) = image_com::send_image(socket, &image_name, &path, ipv4_src, src.port(), 1020, config).await {
                    println!("Image sent successfully!");
                    //let _ = history_table::mark_as_sent(&config.username,&requester_username,&image_name);
                } else {
                    println!("Failed to send image.");
                }
                //image_com::send_image(socket, &image_name, &path, ipv4_src, src.port(), 1020, config).await?;

            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
            // break; there is no breaking man we keep listeening forever and ever and ever
        } 
        else if message.starts_with("GET L ") {
            let image_name = message.trim_start_matches("GET L ").trim().to_string();

            println!("Received low image Request from {}", src);
            if image_name.is_empty() {
                println!("No image name provided");
                continue;
            }
            let response = "ack_request";
            let sending_socket = socket.new_client_socket().await;

            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                com::send(&sending_socket, response.to_string(), (ipv4_src, src.port())).await?;
                let image_path = Path::new(&config.client_raw_images_dir).join(&image_name);
                let low_image_path = Path::new(&config.client_low_quality_images_dir).join(&image_name);
                println!("Creating low image: {}", low_image_path.display());
                println!("input path: {}", image_path.display());
                image_processor::resize_image(image_path.to_str().unwrap(), low_image_path.to_str().unwrap()); //add low image directory
                let low_path = config.client_low_quality_images_dir.clone();
                image_com::send_image(socket, &image_name, &low_path, ipv4_src, src.port(), 1020, config).await?;
            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
            // break; there is no breaking man we keep listeening forever and ever and ever
        } 
        else if message == "GET LIST" {

            let response = "ack_list";

            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                let dest = (ipv4_src, src.port()); // This is the destination address and port

    
                tokio::spawn(async move {
                    
                    if let Err(e) = com::send(&socket::new_client_socket().await, response.to_string(), dest).await {
                        eprintln!("Error sending acknowledgment: {}", e);
                    } else {
                        println!("HERE");
                        if let Err(e) = p2p_send_images_list(&socket::new_client_socket().await, dest).await {
                            eprintln!("Error sending images list: {}", e);
                        }
                    }
                });
            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
        }
            
        
        else {
            println!("Received unexpected message '{}'", message);
        }
    }

    Ok(())
}


pub async fn p2p_send_list_images_request(socket: &Socket, config: &Config, client_address: std::net::Ipv4Addr, message: &String, socket_client_tx_rx: Arc<Mutex<UdpSocket>>) -> std::io::Result<(Arc<Mutex<UdpSocket>>)> {
    let socket_client_tx = socket_client_tx_rx;
    

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

                if response == "ack_list" {
                    //println!("Received acknowledgment '{}' from {}", response, src);
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
        let dest = (client_address, config.port_client_image_request_rx);   
        //println!("Sending 'GET LIST' to {}:{}", dest.0, dest.1);
        com::send(&socket_client_tx, message.to_string(), dest).await?;

        tokio::select! {
            _ = ack_rx.recv() => {
                //println!("Acknowledgment received, moving to next step.");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                println!("Timeout, resending 'GET LIST'...");
            }
        }
    }

    Ok((socket_client_tx))
}

pub async fn p2p_recv_list_images(socket_recv: Arc<Mutex<UdpSocket>>) -> std::io::Result<()> {
    let socket_client_rx = socket_recv.clone();

   
        let (message, _src) = com::recv(&socket_client_rx).await?;
        let message = message.trim();

        // Process the received message, which is expected to be a JSON string.
        match process_json_message(message).await {
            Ok(_) => {
                //println!("Successfully processed message.");
            },
            Err(e) => {
                eprintln!("Error processing message: {}", e);
            }
        }
    

    Ok(())
}

async fn process_json_message(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_json: Value = serde_json::from_str(message)?;

    write_to_file("requested_images.json", message).await?;

    Ok(())
}

async fn write_to_file(filename: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;

    file.write_all(content.as_bytes())?;

    //println!("Successfully wrote to {}", filename);
    Ok(())
}



pub async fn p2p_recv_images_list_request(socket: Arc<Mutex<Socket>>) -> std::io::Result<()> {
    let client_rx = socket.clone();

    
    loop {
        let (message, src) = com::recv(&client_rx.lock().await.socket_client_request_images_rx).await?;
        let message = message.trim();
        if message == "GET LIST" {
            println!("Received 'image Request' from {}", src);

            let response = "ack_list";
            println!("Responding with ack to {}", src);

            if let std::net::IpAddr::V4(ipv4_src) = src.ip() {
                let dest = (ipv4_src, src.port()); // This is the destination address and port

    
                println!("Here");
                tokio::spawn(async move {
                    
                    if let Err(e) = com::send(&socket::new_client_socket().await, response.to_string(), dest).await {
                        eprintln!("Error sending acknowledgment: {}", e);
                    } else {
                        println!("HERE");
                        if let Err(e) = p2p_send_images_list(&socket::new_client_socket().await, dest).await {
                            eprintln!("Error sending images list: {}", e);
                        }
                    }
                });

            } else {
                eprintln!("Received non-IPv4 address: {}", src);
            }
        } else {
            println!("Received unexpected message '{}'", message);
        }
    }

    Ok(())
}



async fn parse_images_file() -> std::io::Result<String> {
    let mut file = File::open("my_images.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let images: ImageList = serde_json::from_str(&contents)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let json_data = serde_json::to_string(&images)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(json_data)
}

pub async fn p2p_send_images_list(socket_send: &Arc<Mutex<UdpSocket>>, dest: (Ipv4Addr, u16)) -> std::io::Result<()> {

    match parse_images_file().await {
        Ok(json_data) => {
            com::send(socket_send, json_data, dest).await?;
            println!("Sent image list to {}:{}", dest.0, dest.1);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error parsing image file: {}", e);
            Err(e)
        }
    }
}






//     Ok(())
// }&config.client_low_images_dir/image_name
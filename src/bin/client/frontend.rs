use async_std::fs::File;
use chrono::format;
use async_std::io::ReadExt;
use serde::Deserialize;

use std::io::{self, Write};
use base64::read;
use mini_redis::client;
use time::convert::Nanosecond;
use tokio::task;
use std::net::{SocketAddr, UdpSocket, Ipv4Addr};
use std::sync::Arc;
use crate::config::Config;
use crate::{api, dos, image_processor, image_store};
use crate::socket::Socket;

    struct Directory {
        hierchy: u32,
        name: String,
        client: Option<Client>, // Add a client field to store the current client
    }
    
    
    pub async fn run() {
        let mut curr_dir = Directory {
            hierchy: 0,
            name: "root".to_string(),
            client: None
        };
        let config = Config::new();
        let socket = Socket::new(config.address_server_1, config.address_server_2, config.address_server_3, config.address_client_leader_rx, config.address_client_tx, config.address_client_rx,config.address_client_dos_tx,config.address_client_dos_rx, config.address_client_image_request_rx).await;
        let config = Config::new();

        let mut rl = rustyline::DefaultEditor::new().expect("Err");
        let socket = Arc::new(socket);
        loop{
            let format_dir = format!("{}>> ", curr_dir.name);
            let readline = rl.readline(format_dir.as_str());
            let socket = socket.clone();
            let config = Config::new();
            
            match readline {
            Ok(line) => match line.as_str() {
                "ls" => {
                    if curr_dir.hierchy == 0 {
                        if let Err(e) = request_print_clients(socket, config).await {
                            println!("Error: {}", e);
                        }
                    }
                    else if curr_dir.hierchy == 1{
                        if let Some(client) = &curr_dir.client {
                            ls_command(socket, config, client).await;
                        }
                    }
                    else {
                        println!("No command");
                    }
                },
                line if line.starts_with("cd") => {
                    let new_dir = line.split(" ").collect::<Vec<&str>>()[1];
                    if new_dir == ".." {
                        if curr_dir.hierchy == 0 {
                            println!("Cannot go back");
                        } else {
                            let new_dir = curr_dir.name.split("/").collect::<Vec<&str>>()[0..curr_dir.hierchy as usize].join("/");
                            curr_dir.name = new_dir;     
                            curr_dir.hierchy -= 1;
                            curr_dir.client = None;
                        }
                    } else {
                        if curr_dir.hierchy == 0 {
                            let client_name = new_dir.to_string();
                            let new_dir = format!("{}/{}", curr_dir.name, new_dir);
                            match load_client(client_name.clone()).await {
                                Ok(client) => {
                                    // Successfully loaded the client
                                    curr_dir.client = Some(client);  // Save the client in `curr_dir`
                                    curr_dir.name = new_dir;  // Update the directory name
                                    curr_dir.hierchy += 1;  // Increase the hierarchy level
                                }
                                Err(e) => {
                                    // Handle the error (e.g., client not found or file errors)
                                    println!("Error loading client: {}", e);
                                }
                            }
                            
                        }
                    }
                },
                line if line.starts_with("see") => {
                    let image_name = line.split(" ").collect::<Vec<&str>>()[1];
                    if curr_dir.hierchy == 1 {
                        if let Some(client) = &curr_dir.client {
                            request_see_image(socket, config, client, image_name, false).await;
                            
                        }
                    }
                },
                line if line.starts_with("request") => {
                    let image_name = line.split(" ").collect::<Vec<&str>>()[1];
                    if curr_dir.hierchy == 1 {
                        if let Some(client) = &curr_dir.client {
                            request_see_image(socket, config, client, image_name, true).await;
                            
                        }
                    }
                },
                "exit" => break,
                _ => println!("No command"),
            },
            Err(_) => println!("No input"),
        }
    }
    
    
}


#[derive(Deserialize, Debug)]
struct Client {
    id: u32,
    ip: String,
    username: String,
}

async fn read_and_print_usernames() -> std::io::Result<()> {
    // Open the file clients.json
    let mut file = File::open("clients_request.json").await?;

    // Read the contents of the file into a string
    let mut contents = String::new();
    use async_std::prelude::*;
    use async_std::io::ReadExt;
    file.read_to_string(&mut contents).await?;

    // Parse the JSON into a vector of Client structs
    let clients: Vec<Client> = serde_json::from_str(&contents).expect("Error parsing JSON");

    // Print the username of each client
    for client in clients {
        println!("{}", client.username);
    }

    Ok(())
}

async fn request_print_clients(socket: Arc<Socket>, config: Config) -> std::io::Result<()> {
    dos::request_dos(&socket, &config).await?;
    read_and_print_usernames().await?;
    Ok(())
}



async fn load_client(client_name: String) -> Result<Client, String> {
    let mut file = match File::open("clients_request.json").await {
        Ok(file) => file,
        Err(e) => return Err(format!("Error opening file: {}", e)),
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents).await {
        return Err(format!("Error reading file: {}", e));
    }

    let clients: Vec<Client> = match serde_json::from_str(&contents) {
        Ok(clients) => clients,
        Err(e) => return Err(format!("Error parsing JSON: {}", e)),
    };

    // Find the client based on the username
    match clients.into_iter().find(|c| {
        let username_trimmed = c.username.trim().to_lowercase();  // Trim and lowercase the client username
        let client_name_trimmed = client_name.trim().to_lowercase();  // Trim and lowercase the input name
        username_trimmed == client_name_trimmed  // Case-insensitive comparison
    }) {
        Some(client) => Ok(client),
        None => Err("Client not found".to_string()),  // Return an error if the client is not found
    }
}


async fn ls_command(socket: Arc<Socket>, config: Config, client: &Client) {
    // Parse the client IP string into four u8 values
    let ip_parts: Vec<u8> = client.ip
        .split('.')
        .map(|part| part.parse::<u8>().expect("Invalid IP address octet"))
        .collect();
    if ip_parts.len() == 4 {
        let client_ip = Ipv4Addr::new(ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]);
        // Now you can pass the four u8 values to the API function
        api::request_list_images(&socket, &config, client_ip).await.unwrap();
        //read and print the images
        image_store::display_images_data("requested_images.json");
        
    } else {
        panic!("Invalid IP address format: {}", client.ip);
    }
}


async fn request_see_image(socket: Arc<Socket>, config: Config, client: &Client, image_name: &str, is_high: bool) {
    // Parse the client IP string into four u8 values
    let ip_parts: Vec<u8> = client.ip
        .split('.')
        .map(|part| part.parse::<u8>().expect("Invalid IP address octet"))
        .collect();
    if ip_parts.len() == 4 {
        let client_ip = Ipv4Addr::new(ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]);
        // Now you can pass the four u8 values to the API function
        let sending_socket = socket.new_client_socket().await;
        let client_port = config.port_client_image_request_rx;
        api::request_image(&socket, &config, sending_socket, image_name.to_string(), client_ip, client_port, is_high).await.unwrap();
        let path_input = config.client_low_quality_images_dir + "/"+ image_name;
        if !is_high{
            image_processor::display_image(&path_input);
        }
        
    } else {
        panic!("Invalid IP address format: {}", client.ip);
    }
}

async fn view_image(path: &str){
    // get views left
    // check number of views left if greater than 0
    // decode
    // decrement views
    // display
    image_processor::display_image(path);
}
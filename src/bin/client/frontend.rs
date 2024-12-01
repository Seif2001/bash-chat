use async_std::fs::{self, File};
use async_std::path::Path;
use bincode::config;
use chrono::format;
use async_std::io::ReadExt;
use serde::Deserialize;

use std::io::{self, Write};
use base64::{display, read};
use mini_redis::client;
use time::convert::Nanosecond;
use tokio::task;
use std::net::{SocketAddr, UdpSocket, Ipv4Addr};
use std::sync::Arc;
use crate::config::Config;
use crate::dos::get_ip_by_username;
use crate::{api, dos, image_processor, image_store, middleware};
use crate::socket::Socket;
use crate::history_table;

    struct Directory {
        hierchy: u32,
        name: String,
        client: Option<Client>, // Add a client field to store the current client
        
    }
    
    
    pub async fn run(socket: &Socket, config: Config) {
        let mut curr_dir = Directory {
            hierchy: 0,
            name: "root".to_string(),
            client: None
        };
 
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
                //1 && curr_dir.name.starts_with("clients")
                "ls" => {
                    if curr_dir.hierchy == 0 {
                        println!("clients");
                        println!("resources");
                        println!("current_requests");
                    }
                    else
                    if curr_dir.hierchy == 1 {
                        if curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "clients"{
                            if let Err(e) = request_print_clients(socket, config).await {
                                println!("Error: {}", e);
                            }
                        }
                        else if curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "resources"{
                            println!("my_images");
                            println!("requested_images");
                        }
                        else if curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "current_requests"{
                            history_table::read_and_print_unapproved_requests().await;
                        }
                        
                    }
                    else if curr_dir.hierchy == 2 && curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "clients"{
                        if let Some(client) = &curr_dir.client {
                            ls_command(socket, config, client).await;
                        }
                    }
                    else if curr_dir.hierchy == 2 && curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "resources"{
                        if curr_dir.name.split("/").collect::<Vec<&str>>()[2] == "my_images"{
                            display_images_data(config.client_raw_images_dir).await;
                        }
                        else if curr_dir.name.split("/").collect::<Vec<&str>>()[2] == "requested_images"{
                            display_images_data(config.client_high_quality_receive_dir).await;
                        }
                    }
                    else {
                        println!("No command");
                    }
                    // else if curr_dir.hierchy == 0{
                    //     println!("resources")
                    // }
                },
                line if line.starts_with("view") => {
                    if curr_dir.hierchy == 2 && curr_dir.name.split("/").collect::<Vec<&str>>()[2] == "my_images"{
                        let image_name = line.split(" ").collect::<Vec<&str>>()[1];
                        let path = config.client_raw_images_dir + "/" + image_name;
                        view_image(path).await;
                    }
                    else if curr_dir.hierchy == 2 && curr_dir.name.split("/").collect::<Vec<&str>>()[2] == "requested_images"{
                        
                        let image_name = line.split(" ").collect::<Vec<&str>>()[1];
                        let path = config.client_high_quality_receive_dir + "/" + image_name;
                        view_hq_image(path).await;
                        
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
                            let next_dir = new_dir.to_string();
                            if next_dir == "clients" || next_dir == "resources" {
                                let new_dir = format!("{}/{}", curr_dir.name, new_dir);
                                curr_dir.name = new_dir;
                                curr_dir.hierchy += 1;
                                
                            }
                            else if next_dir == "current_requests"{
                                let new_dir = format!("{}/{}", curr_dir.name, new_dir);
                                curr_dir.name = new_dir;
                                curr_dir.hierchy += 1;
                            }
                        }
                        else
                        if curr_dir.hierchy == 1 && curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "clients" {
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
                        else if curr_dir.hierchy == 1 && curr_dir.name.split("/").collect::<Vec<&str>>()[1] == "resources" {
                            let next_dir = new_dir.to_string();
                            if next_dir == "my_images" || next_dir == "requested_images" {
                                let new_dir = format!("{}/{}", curr_dir.name, new_dir);
                                curr_dir.name = new_dir;
                                curr_dir.hierchy += 1;
                                
                            }
                        }
                    }
                },
                line if line.starts_with("see") => {
                    let image_name = line.split(" ").collect::<Vec<&str>>()[1];
                    if curr_dir.hierchy == 2 {
                        if let Some(client) = &curr_dir.client {
                            request_see_image(socket, config, client, image_name, false).await;
                            
                        }
                    }
                },
                line if line.starts_with("request") => {
                    if line.len() > 0{

                        let image_name = line.split(" ").collect::<Vec<&str>>()[1];
                        if curr_dir.hierchy == 2 {
                            if let Some(client) = &curr_dir.client {
                                request_see_image(socket, config, client, image_name, true).await;
                                
                            }
                        }
                    }
                },
                line if line.starts_with("approve") => {
                    let req_number = line.split(" ").collect::<Vec<&str>>()[1];
                    
                    match history_table::get_approved_request_by_number(req_number.parse::<usize>().unwrap()).await {
                        Ok((image_name, client_name, client_port)) => {
                            let client_ip = get_ip_by_username(client_name.as_str()).unwrap();
                            let client_ip = client_ip.parse::<Ipv4Addr>().expect("Invalid IP address format");
                            middleware::send_image(&*socket, &image_name, client_ip, client_port, 1020, &config).await.unwrap();
                        }
                        Err(e) => {
                            println!("Error: {}", e);
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

async fn read_and_print_usernames(config: Config) -> std::io::Result<()> {
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
        if client.username != config.username {
            println!("{}", client.username);
        }
    }

    Ok(())
}

async fn request_print_clients(socket: Arc<&Socket>, config: Config) -> std::io::Result<()> {
    dos::request_dos(&socket, &config).await?;
    read_and_print_usernames(config).await?;
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


async fn ls_command(socket: Arc<&Socket>, config: Config, client: &Client) {
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


async fn request_see_image(socket: Arc<&Socket>, config: Config, client: &Client, image_name: &str, is_high: bool) {
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

async fn view_image(path: String){
    //check if the path is valid
    let file_path = Path::new(&path);

    if file_path.exists().await {
        if file_path.is_file().await {
            // If the file exists, proceed to display the image
            image_processor::display_image(&path.clone());
        } else {
            println!("Error: Path is not a file.");
        }
    } else {
        println!("Error: Image file not found at {}", path);
    }
}

async fn view_hq_image(path: String){
    //check if the path is valid
    let file_path = Path::new(&path);

    if file_path.exists().await {
        if file_path.is_file().await {
            // If the file exists, proceed to display the image
            let views = image_processor::get_views(path.clone());
            println!("Views: {:?}", views);
            if let Ok(views) = views {
                if views > 0{
                    let image = image_processor::decode_image_no_save(path.clone());
                    image_processor::display_image_no_save(image);
                    image_processor::append_views(path.clone(), path.clone(), views-1);
                }
                else{
                    fs::remove_file(path.clone()).await.unwrap();
                    println!("Error: Image has no views left.");
                }
            }
        } else {
            println!("Error: Path is not a file.");
        }
    } else {
        println!("Error: Image file not found at {}", path);
    }
}


async fn display_images_data(path: String) {
    // Check if the path is valid
    let path = Path::new(&path);

    // Attempt to read the directory
    match fs::read_dir(path).await {
        Ok(entries) => {
            // Iterate through the entries in the directory
            use async_std::stream::StreamExt;
            let mut entries = entries;
            while let Some(entry) = entries.next().await {
                match entry {
                    Ok(entry) => {
                        let file_name = entry.file_name();
                        let file_path = entry.path();

                        // Check if the file has an image extension (e.g., .jpg, .png)
                        if let Some(extension) = file_path.extension() {
                            let ext = extension.to_string_lossy().to_lowercase();
                            if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext.as_str()) {
                                println!("{:}", file_name.to_string_lossy());
                            }
                        }
                    }
                    Err(e) => println!("Error reading entry: {}", e),
                }
            }
        }
        Err(e) => {
            println!("Failed to read directory {}: {}", path.display(), e);
        }
    }
}

pub async fn get_username_by_ip(ip: &str) -> Result<String, std::io::Error> {
    let json_path = "clients_request.json";
    let json_data = fs::read_to_string(json_path).await?;

    let clients: Vec<Client> = serde_json::from_str(&json_data)?;

    for client in clients {
        if client.ip == ip {
            return Ok(client.username);
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "IP address not found"))
}

pub async fn approve_request(image_name: &str, client_ip: Ipv4Addr) -> io::Result<bool> {
    let user_requester = get_username_by_ip(&client_ip.to_string()).await?;
    println!("User: {} wants to view image: {} [Y/n]", user_requester, image_name);
    let mut input = String::new();
    async_std::io::stdin().read_line(&mut input).await.expect("Failed to read line");

    // Trim and check user input
    Ok(input.trim().to_lowercase() == "y")
    // Add your logic here to determine if the request is approved
    // For example, you can return Ok(true) if approved, or Ok(false) if not approved
}
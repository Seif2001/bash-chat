use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use crate::leader::{self, Node};


use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_reader, to_writer};
use std::io::Write;
use std::io::Read;
use tokio::time::{timeout, Duration};
use crate::config::{Config};
use crate::socket::{self, Socket};
use crate::com;


#[derive(Serialize, Deserialize)]
struct ClientInfo {
    id: u32,
    ip: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HistoryEntry {
    request_from: String,
    request_to: String,
    image: String,
}


pub async fn find_leader(servers: Arc<Mutex<HashMap<u32, Node>>>)->u32{
    let mut leader_id: u32 = 0;
    let db = servers.lock().await;

    for (key, node) in db.iter(){
        if node.is_leader {
            leader_id = *key;
        }
    }
    drop(db); // Unlock the mutex before locking it again

    leader_id
}

pub async fn read_file(message: String) -> std::io::Result<()> {
    let file_path = "clients.json";

    let mut file = File::create(file_path)?;
    file.write_all(message.as_bytes())?;

    Ok(())
}
pub async fn send_ack( socket: &Socket, config: &Config, client_addr: Ipv4Addr) -> std::io::Result<()> {
    let dest = (client_addr, config.port_client_dos_rx);    
    let socket = socket.socket_client_dos_rx.clone();

    com::send(&socket, "ACK".to_string(), dest).await
}


pub async fn send_ack_random(client_addr: Ipv4Addr,client_port:u16) -> std::io::Result<()> {
    let dest = (client_addr, client_port);
    let socket = &socket::new_server_socket().await;
    com::send(&socket, "ACK".to_string(), dest).await
}


pub async fn send_dos( socket: &Socket, config:&Config) -> std::io::Result<()> {
    let file_path = "clients.json";
    let file_content = match File::open(file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        }
        Err(e) => {
            eprintln!("Failed to read the file: {}", e);
            return Err(e);
        }
    };
    let dest = (config.multicast_addr, config.port_server_dos_rx);    
    let socket = socket.socket_server_dos_rx.clone();


    com::send(&socket, file_content, dest).await
    
}





pub async fn send_dos_client( socket: &Socket, config: &Config, client_addr: Ipv4Addr) -> std::io::Result<()> {
    let file_path = "clients.json";
    let file_content = match File::open(file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        }
        Err(e) => {
            eprintln!("Failed to read the file: {}", e);
            return Err(e);
        }
    };
    let dest = (client_addr, config.port_client_dos_rx);    
    let socket = socket.socket_client_dos_rx.clone();

    com::send(&socket, file_content, dest).await
}


pub async fn update_dos(client_addr: Ipv4Addr, username: String) {
    let file_path = "clients.json";

    // Create a vector of clients using the json file or create a new vector if there's no json file
    let mut clients = if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap_or_else(|_| Vec::<ClientInfo>::new())
    } else {
        Vec::<ClientInfo>::new()
    };

    if let Some(client) = clients.iter_mut().find(|client| client.username == username) {
        client.ip = client_addr.to_string(); //if  exists update ip
    } else { //other wise add and push
        let next_id = clients.iter().map(|client| client.id).max().unwrap_or(0) + 1;

        let new_client = ClientInfo {
            id: next_id,
            ip: client_addr.to_string(),
            username,
        };

        clients.push(new_client);
    }

    let file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path).expect("Unable to open or create the file");
    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &clients).expect("Failed to write to file");
}

pub async fn dos_registrar(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id: u32, socket: &Arc<Socket>, config: &Arc<Config>) {
    let servers_clone = Arc::clone(&servers);
    let socket_client = socket.socket_client_dos_tx.clone();
    let servers = Arc::clone(&servers_clone);
    let socket = Arc::clone(&socket);
    let config = Arc::clone(config);

    tokio::spawn(async move {
        loop {
            println!("Waiting for dos message from client");
            let (message, client_addr) = com::recv(&socket_client).await.expect("Failed to receive message");
            let message = message.trim();
            let client_port:u16 = client_addr.port();
            let client_addr = match client_addr.ip() {
                std::net::IpAddr::V4(addr) => addr,
                _ => panic!("Expected an IPv4 address"),
            };
            if message.starts_with("REGISTER ") {
                println!("Received dos message from client");
                let username = message.trim_start_matches("REGISTER ").trim().to_string();
                if username.is_empty() {
                    println!("No username provided with REGISTER message. Ignoring.");
                    continue;
                }
                let servers_clone = Arc::clone(&servers);
                if find_leader(servers_clone).await == my_id{
                    update_dos(client_addr, username).await;
                    let socket_clone = Arc::clone(&socket);
                    let config_clone = Arc::clone(&config);
                    let _ = send_dos(&socket_clone,&config_clone).await;
                    println!("Sent dos to other servers");
                    let _ = send_ack(&socket_clone, &config_clone,client_addr).await;
                }
            }
            else if message == "REQUEST" {
                let servers_clone = Arc::clone(&servers);
                if find_leader(servers_clone).await == my_id{
                    let socket_clone = Arc::clone(&socket);
                    let config_clone = Arc::clone(&config);
                    let _ = send_dos_client(&socket_clone,&config_clone,client_addr).await;
                    println!("Sent dos to client");
                }
            }
            else if message.starts_with("UPDATE ") {
                println!("Received update message from client");
                let data = message.trim_start_matches("UPDATE ").trim().to_string();
                if data.is_empty() {
                    println!("No data provided ignoring");
                    continue;
                }
                let servers_clone = Arc::clone(&servers);
                if find_leader(servers_clone).await == my_id{
                    update_history_table(data).await;
                    let socket_clone = Arc::clone(&socket);
                    let config_clone = Arc::clone(&config);
                    let _ = send_history(&socket_clone, &config_clone);
                    let _ = send_ack_random(client_addr,client_port).await;
                }
            }
            else if message.starts_with("DELETE ") {
                println!("Received delete message from client");
                let data = message.trim_start_matches("DELETE ").trim().to_string();
                if data.is_empty() {
                    println!("No data provided ignoring");
                    continue;
                }
                let servers_clone = Arc::clone(&servers);
                if find_leader(servers_clone).await == my_id{
                    delete_history_table(data).await;
                    let socket_clone = Arc::clone(&socket);
                    let config_clone = Arc::clone(&config);
                    let _ = send_history(&socket_clone, &config_clone);
                    let _ = send_ack_random(client_addr,client_port).await;
                }
            }      
        }
    });
}

pub async fn recv_dos(socket: &Arc<Socket>, config: &Arc<Config>) {
    println!("Waiting for dos file updates");

    let server_ip: Ipv4Addr = config.interface_addr;
    let socket_dos_rx = socket.socket_server_dos_rx.clone();
    
    tokio::spawn(async move {
        loop {
            let result = timeout(Duration::from_secs(1), com::recv(&socket_dos_rx)).await; // stop every once sec to release socket

            match result {
                Ok(Ok((message, src))) => {
                    let message = message.trim();
                    if src.ip() == server_ip {
                        println!("Ignoring message from own IP: {}", src.ip());
                        continue;
                    }
                    if message.starts_with("DOS") {
                        println!("Received DOS file update");
                        let file_content = message.trim_start_matches("DOS").trim().to_string();                        
                        if let Err(e) = read_file(file_content).await {
                            eprintln!("Failed to process received file: {}", e);
                        }
                    } else if message.starts_with("HISTORY") {
                        let file_content = message.trim_start_matches("HISTORY").trim().to_string();                        
                        if let Err(e) = read_file(file_content).await {
                            eprintln!("Failed to process received file: {}", e);
                        }
                    } else {
                        println!("Received unrecognized message: {}", message);
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to receive message: {}", e);
                }
                Err(_) => {
                    // Timeout reached, no message received
                }
            }
        }
    });
}

//history table 

pub async fn update_history_table(data: String) {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() != 3 {
        eprintln!("Invalid update data format. Expected: from_username to_username image_name");
        return;
    }

    let from_username = parts[0].to_string();
    let to_username = parts[1].to_string();
    let image_name = parts[2].to_string();

    let new_entry = HistoryEntry {
        request_from: from_username.clone(),
        request_to: to_username.clone(),
        image: image_name.clone(),
    };

    let file_path = "history.json";
    let mut history = if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        from_reader(reader).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    history.push(new_entry);

    let file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)
        .expect("Unable to open or create the file");
    let writer = BufWriter::new(file);
    to_writer(writer, &history).expect("Failed to write to file");
}

pub async fn delete_history_table(data: String) {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() != 3 {
        eprintln!("Invalid delete data format. Expected: from_username to_username image_name");
        return;
    }

    let from_username = parts[0].to_string();
    let to_username = parts[1].to_string();
    let image_name = parts[2].to_string();

    let file_path = "history.json";

    // Read the existing history from the file
    let mut history = if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        from_reader(reader).unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // Explicitly specify that `entry` is of type `HistoryEntry` in the closure
    history.retain(|entry: &HistoryEntry| {
        !(entry.request_from == from_username
            && entry.request_to == to_username
            && entry.image == image_name)
    });

    // Write the updated history back to the file
    let file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)
        .expect("Unable to open or create the file");
    let writer = BufWriter::new(file);
    to_writer(writer, &history).expect("Failed to write to file");

    println!("Deleted entry with from_username: {}, to_username: {}, image_name: {}", from_username, to_username, image_name);
}

pub async fn send_history( socket: &Socket, config:&Config) -> std::io::Result<()> {
    let file_path = "history.json";
    let file_content = match File::open(file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        }
        Err(e) => {
            eprintln!("Failed to read the file: {}", e);
            return Err(e);
        }
    };
    let dest = (config.multicast_addr, config.port_server_dos_rx);    
    let socket = socket.socket_server_dos_rx.clone();


    com::send(&socket, file_content, dest).await
    
}


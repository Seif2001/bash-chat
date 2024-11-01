use tokio::net::UdpSocket;
use std::fmt::format;
use std::net::Ipv4Addr;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use tokio::time::{self, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use sysinfo::{System};
use crate::middleware;

pub struct Server {
    id: u32,
    failed: bool,
    leader: Arc<RwLock<u32>>,
    socket_client: Arc<UdpSocket>, // for client to server communication
    socket_server: Arc<UdpSocket>, // for server to server communication
    multicast_addr: Ipv4Addr,
    interface_addr: Ipv4Addr,
    port_server: u16,
    db: Arc<Mutex<HashMap<u32, u32>>>, // Wrap db in Arc<Mutex> for thread-safe access
}

impl Server {
    pub async fn new(id: u32, failed: bool, leader: Arc<RwLock<u32>>) -> Server {
        dotenv().ok();
        
        let port_server = env::var("PORT_SERVER").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_client = env::var("PORT_CLIENT").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");

        // Define server addresses
        let server_ip = "0.0.0.0:";
        let server_address_server = format!("{}{}", server_ip, port_server);
        let server_address_client = format!("{}{}", server_ip, port_client);

        // Bind server and client sockets asynchronously
        let socket_server = Arc::new(UdpSocket::bind(server_address_server).await.expect("Failed to bind server socket"));
        let socket_client = Arc::new(UdpSocket::bind(server_address_client).await.expect("Failed to bind client socket"));

        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");
        let db = Arc::new(Mutex::new(HashMap::new())); // Initialize db as Arc<Mutex>



        Server {
            id,
            failed,
            leader,
            socket_server,
            socket_client,
            multicast_addr,
            interface_addr,
            port_server,
            db,
        }
    }

    pub async fn join(self:Arc<Self>) -> std::io::Result<()> {        
        middleware::join(self.socket_server.clone(), self.interface_addr, self.multicast_addr)
            .await
            .expect("Failed to join multicast group");
        Ok(())
    }

    pub async fn election(self: Arc<Self>) -> std::io::Result<()> {
        // Start sending leader messages
        
        let send_leader_future = self.clone().send_leader();
        
        let send_info_future = self.clone().send_info();
        // Start receiving leader messages
        let recv_info_future = self.clone().recv_info();
    
        // Start sending system information
    
        // Use tokio::select! to run the futures concurrently
        // Use tokio::join! to run the futures concurrently
        let (send_leader_result, recv_info_future, send_info_result) = tokio::join!(
            send_leader_future,
            recv_info_future,
            send_info_future,
        );
    
        Ok(())
    }
    

    pub async fn send_leader(self:Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_server;
        let id = self.id;
    
        // Spawn a new async task for sending leader messages
        tokio::spawn(async move {
            let message = format!("Lead:{}", id.clone());
            
            // Continue sending messages while not the leader
            while *self.leader.read().expect("Failed to read leader") == id {
                println!("Sending message: {}", message);
                match middleware::send_message(socket_server.clone(), multicast_addr, port_server, message.clone()).await {
                    Ok(_) => println!("Message sent successfully"),
                    Err(e) => eprintln!("Failed to send RPC: {}", e),
                }
    
                // Sleep for 1 second before sending the next message
                time::sleep(Duration::from_secs(1)).await;
            }
        });
    
        Ok(())
    }
    

    // pub async fn recv_leader(self: Arc<Self>) {
    //     loop {
    //         let mut buf = vec![0u8; 1024];
    //         let socket_server = self.socket_server.clone();
    //         let leader_lock = self.leader.clone(); // Clone Arc<RwLock<u32>> for async move
    
    //         // Try to receive messages
    //         match socket_server.recv_from(&mut buf).await {
    //             Ok((len, addr)) => {
    //                 let data = buf[..len].to_vec();
    //                 let message = String::from_utf8_lossy(&data).to_string();
    //                 println!("Received '{}' from {}", message, addr);
    
    //                 // Spawn an async task to handle each message
    //                 tokio::spawn(async move {
    //                     // Check if the message is in the correct format "Lead: id"
    //                     if let Some(id_str) = message.strip_prefix("Lead:") {
    //                         // Parse the id from the message
    //                         if let Ok(id) = id_str.trim().parse::<u32>() {
    //                             let mut leader = leader_lock.write().expect("Failed to write leader");
    //                             *leader = id;
    //                             println!("Updated leader to '{}' from {}", id, addr);
    //                         } else {
    //                             eprintln!("Failed to parse leader id from message: {}", message);
    //                         }
    //                     } else {
    //                         println!("Received invalid message format from {}", addr);
    //                     }
    //                 });
    //             }
    //             Err(e) => {
    //                 eprintln!("Failed to receive message: {}", e);
    //             }
    //         }
    //     }
    // }
    pub async fn recv_info(self: Arc<Self>) -> std::io::Result<()> {
        loop {
            let mut buf = vec![0u8; 1024];
            let socket_server = self.socket_server.clone();
            let db = self.db.clone(); // Clone Arc<Mutex<HashMap<u32, u32>>> for async move
            let leader_lock = self.leader.clone(); // Clone Arc<RwLock<u32>> for async move
            
            // Try to receive messages
            match socket_server.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    let data = buf[..len].to_vec();
                    let message = String::from_utf8_lossy(&data).to_string();
                    println!("Received '{}' from {}", message, addr);
                    
                    // Spawn an async task to handle each message
                    tokio::spawn({
                        let db = db.clone();
                        let leader_lock = leader_lock.clone();
                        let self_clone = self.clone();
    
                        async move {
                            // Check if the message is in the correct format "Lead: id"
                            if let Some(id_str) = message.strip_prefix("Lead:") {
                                // Parse the id from the message
                                if let Ok(id) = id_str.trim().parse::<u32>() {
                                    let mut leader = leader_lock.write().expect("Failed to write leader");
                                    *leader = id;
                                    println!("Updated leader to '{}' from {}", id, addr);
                                } else {
                                    eprintln!("Failed to parse leader id from message: {}", message);
                                }
                            } else if let Some((key, value)) = message.split_once(':') {
                                // Handle the key:value format
                                let key = key.trim();
                                if let Ok(parsed_key) = key.parse::<u32>() {
                                    if let Ok(value) = value.trim().parse::<u32>() {
                                        let mut db_lock = db.lock().unwrap();
                                        db_lock.insert(parsed_key, value);
    
                                        if let Ok(Some(new_leader)) = self_clone.choose_leader() {
                                            let mut leader = leader_lock.write().expect("Failed to write leader");
                                            *leader = new_leader;
                                            println!("Updated database with {}:{} new leader: {}", parsed_key, value, new_leader);
                                        }
                                    } else {
                                        eprintln!("Failed to parse value from message: {}", message);
                                    }
                                } else {
                                    eprintln!("Failed to parse key from message: {}", message);
                                }
                            } else {
                                println!("Received invalid message format from {}", addr);
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to receive message: {}", e);
                }
            }
        }
    }
    
    
    
    

    pub fn choose_leader(self:Arc<Self>) -> std::io::Result<Option<u32>> {
        // Initialize variables to store the minimum value and the corresponding key
        let mut min_key: Option<u32> = None;
        let mut min_value: Option<u32> = None;
        let leader_lock: Arc<RwLock<u32>> = self.leader.clone(); // Clone Arc<RwLock<u32>> for async move

    
        // Iterate through the entries in the HashMap
        let min_key = {
            let db = self.db.lock().unwrap();
            for (&key, &value) in db.iter() {
                // Check if we haven't set a minimum value yet or if the current value is lower than the minimum found
                if min_value.is_none() || value < min_value.unwrap() {
                    min_value = Some(value); // Update the minimum value
                    min_key = Some(key);     // Update the corresponding key
                }
            }
            min_key
        };
    
        // Return the key with the lowest value, or None if the map is empty
        
        Ok(min_key) // Set the leader to the server with the lowest value or the current server
    
    }

    pub async fn send_info(self:Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_server;
        let id = self.id;
        print!("inside send_info"); 
        // Spawn a new async task for sending info
        tokio::spawn(async move {
            let mut sys = System::new_all();

            loop {
                sys.refresh_all(); // Refresh system information

                // Get memory and CPU usage
                let total_memory = sys.total_memory();
                let used_memory = sys.used_memory();
                let cpu_usage = sys.global_cpu_usage();

                // Update shared CPU and memory usage
                let cpu = cpu_usage as u32; // Update CPU usage
                let mem = (used_memory as f32 / total_memory as f32 * 100.0) as u32; // Update memory usage

                let message = format!("{}:{}", id, cpu * mem);
                
                println!("Sending message: {}", message);
                middleware::send_message(socket_server.clone(), multicast_addr, port_server, message)
                    .await
                    .expect("Failed to send RPC");

                time::sleep(Duration::from_secs(1)).await; // Sleep for 1 second
            }
        });
        Ok(())
    }

    

    // pub async fn process_client(self:Arc<Self>) -> std::io::Result<()> {
    //     loop {
    //         let db = self.db.clone();
    //         let socket_client = self.socket_client.clone();
    //         let mut buf = vec![0u8; 1024];
    //         let (len, addr) = self.socket_client.recv_from(&mut buf).await.expect("Failed to receive from client socket");
    //         let data = buf[..len].to_vec();
    
    //         tokio::spawn(async move {
    //             middleware::process(socket_client, db, data, addr).await;
    //         });
    //     }

    //     // Optionally add client processing logic here
    // }
}

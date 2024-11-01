use tokio::net::UdpSocket;
use std::fmt::format;
use std::net::Ipv4Addr;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use tokio::time::{self, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sysinfo::{System};
use crate::middleware;

pub struct Server {
    id: u32,
    failed: bool,
    leader: u32,
    socket_client: Arc<UdpSocket>, // for client to server communication
    socket_server: Arc<UdpSocket>, // for server to server communication
    multicast_addr: Ipv4Addr,
    interface_addr: Ipv4Addr,
    port_server: u16,
    db: Arc<Mutex<HashMap<u32, u32>>>, // Wrap db in Arc<Mutex> for thread-safe access
}

impl Server {
    pub async fn new(id: u32, failed: bool, leader: u32) -> Server {
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
        self.send_info();
        Ok(())
    }

    // pub async fn election(self: Arc<Self>) -> std::io::Result<()>{
    //     let self_clone = self.clone();
    //     self.send_info().await?;
    //     self_clone.send_leader().await?;
    //     Ok(())
    // }

    // pub async fn recv_info(self: Arc<Self>) -> std::io::Result<()> {
    //     loop {
    //         // Spawn a new task for receiving messages
    //         let socket_server = self.socket_server.clone();
    //         let db = self.db.clone(); // Clone the Arc<Mutex<HashMap<u32, u32>>>
    
    //         tokio::spawn(async move {
    //             let result = middleware::recv_rpc(socket_server.clone()).await;
    
    //             match result {
    //                 Ok(message) => {
    //                     // Split the message into id and value
    //                     let parts: Vec<&str> = message.split(':').collect();
    //                     if parts.len() == 2 {
    //                         if let (Ok(id), Ok(value)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
    //                             // Lock the database and update the value
    //                             let mut db = db.lock().unwrap();
    //                             db.insert(id, value);
    //                             println!("Updated db with id: {} and value: {}", id, value);
    //                         } else {
    //                             eprintln!("Failed to parse id or value from message: {}", message);
    //                         }
    //                     }
    //                 },
    //                 Err(e) => eprintln!("Failed to receive RPC: {}", e),
    //             }
    //         });
    //     }
    
    //     Ok(())
    // }
    

    // pub fn choose_leader(mut self:Arc<Self>) -> std::io::Result<()> {
    //     // Initialize variables to store the minimum value and the corresponding key
    //     let mut min_key: Option<u32> = None;
    //     let mut min_value: Option<u32> = None;
    
    //     // Iterate through the entries in the HashMap
    //     let min_key = {
    //         let db = self.db.lock().unwrap();
    //         for (&key, &value) in db.iter() {
    //             // Check if we haven't set a minimum value yet or if the current value is lower than the minimum found
    //             if min_value.is_none() || value < min_value.unwrap() {
    //                 min_value = Some(value); // Update the minimum value
    //                 min_key = Some(key);     // Update the corresponding key
    //             }
    //         }
    //         min_key
    //     };
    
    //     // Return the key with the lowest value, or None if the map is empty
    //     let mut server = Arc::get_mut(&mut self).expect("Failed to get mutable reference to server");
    //     server.leader = min_key.unwrap();
    //     Ok(())
    // }

    pub async fn send_info(self:Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_server;
        let id = self.id;
        
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
                middleware::send_rpc(socket_server.clone(), multicast_addr, port_server, message)
                    .await
                    .expect("Failed to send RPC");

                time::sleep(Duration::from_secs(1)).await; // Sleep for 1 second
            }
        });
        Ok(())
    }

    pub async fn send_leader(self:Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_server;
        let id = self.id;
    
        // Spawn a new async task for sending leader messages
        tokio::spawn(async move {
            let message = format!("Lead: {}", id.clone());
            
            // Continue sending messages while not the leader
            while self.leader == id {
                println!("Sending message: {}", message);
                match middleware::send_rpc(socket_server.clone(), multicast_addr, port_server, message.clone()).await {
                    Ok(_) => println!("Message sent successfully"),
                    Err(e) => eprintln!("Failed to send RPC: {}", e),
                }
    
                // Sleep for 1 second before sending the next message
                time::sleep(Duration::from_secs(1)).await;
            }
        });
    
        Ok(())
    }
    

    

    pub async fn send_rpc(&self) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_server;

        let id = self.id;
        let message = id.to_string();

        tokio::spawn(async move {
            middleware::send_rpc(socket_server.clone(), multicast_addr, port_server, message)
                .await
                .expect("Failed to send RPC");
            time::sleep(Duration::from_secs(1)).await;
        });
        Ok(())
    }

    pub async fn recv_rpc(&self) {
        loop{
            let mut buf = vec![0u8; 1024];
            let socket_server = self.socket_server.clone();
            let (len, addr) = socket_server.recv_from(&mut buf).await.expect("Failed");
            let data = buf[..len].to_vec();
            let message = String::from_utf8(data).unwrap();

            tokio::spawn(async move{

                println!("Recieved message : {} from {}", message, addr);
            });
        }
        
    }

    // pub async fn process_client(&self) -> std::io::Result<()> {
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

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

    pub async fn join(&self) -> std::io::Result<()> {        
        middleware::join(self.socket_server.clone(), self.interface_addr, self.multicast_addr)
            .await
            .expect("Failed to join multicast group");

        self.send_leader().await?;

        Ok(())
    }

    pub async fn send_info(&self) -> std::io::Result<()> {
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

                let message = format!("{}:{}:{}:{}", id, cpu, mem, 0);
                
                println!("Sending message: {}", message);
                middleware::send_rpc(socket_server.clone(), multicast_addr, port_server, message)
                    .await
                    .expect("Failed to send RPC");

                time::sleep(Duration::from_secs(1)).await; // Sleep for 1 second
            }
        });
        Ok(())
    }

    pub async fn send_leader(self) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_server;
        let id = self.id;
    
        // Spawn a new async task for sending leader messages
        tokio::spawn(async move {
            let message = format!("Lead: {}", id.clone());
            
            // Continue sending messages while not the leader
            while self.leader != id {
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

    pub async fn recv_rpc(&self) -> std::io::Result<()> {
        loop{tokio::spawn({
            let socket_server = self.socket_server.clone();

            async move {
                
                    middleware::recv_rpc(socket_server.clone()).await.expect("Failed to receive RPC");
            }   
        });}
        
        Ok(())
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

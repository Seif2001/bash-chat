use tokio::net::UdpSocket;
use std::fmt::format;
use std::net::Ipv4Addr;
use dotenv::dotenv;
use std::{clone, env};
use std::str::FromStr;
use tokio::time::{self, Duration,timeout};
use std::collections::HashMap;
use tokio::sync::Mutex; // Use tokio's async Mutex
use std::sync::Arc;
use sysinfo::{System};
use crate::image_processor;
use tokio::signal;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::fs::File;
use crate::middleware;
use std::sync::{RwLock};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};


pub struct Server {
    id: u32,
    failed: Arc<RwLock<bool>>,
    leader: Arc<AtomicBool>,     // Changed from RwLock to AtomicBool
    election: Arc<AtomicBool>,   // Changed from RwLock to AtomicBool
    socket_client_send: Arc<UdpSocket>, // for client to server communication
    socket_leader: Arc<UdpSocket>, // for server to server communication
    socket_election: Arc<UdpSocket>, // for server to server communication
    socket_bully: Arc<UdpSocket>, // for bully algorithm
    socket_server:  Arc<UdpSocket>, // for bully algorithm
    socket_server_recv:  Arc<UdpSocket>, // for bully algorithm
    multicast_addr: Ipv4Addr,
    interface_addr: Ipv4Addr,
    port_leader: u16,
    socket_server_leader: Arc<UdpSocket>,
    port_election: u16,
    port_bully: u16,
    port_server_recv: u16,
    db: Arc<Mutex<HashMap<u32, u32>>>, // Wrap db in Arc<Mutex> for thread-safe access
    random_number: Arc<Mutex<u32>>,
    client_address: Arc<Mutex<Ipv4Addr>>,
}


impl Server {
    pub async fn new(id: u32, failed: bool, leader: Arc<AtomicBool>, election: Arc<AtomicBool>) -> Server {
        dotenv().ok();
        
        let port_leader = env::var("PORT_LEADER").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_election = env::var("PORT_ELECTION").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_client = env::var("PORT_CLIENT").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");
        let port_bully = env::var("PORT_BULLY").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
        let port_server = env::var("PORT_SERVER").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_server_recv = env::var("PORT_SERVER_RECV").expect("PORT_SERVER_RECV not set").parse::<u16>().expect("Invalid server port");
        let port_server_leader = env::var("PORT_SERVER_LEADER").expect("PORT_SERVER_LEADER not set").parse::<u16>().expect("Invalid server port");
    


        // Define server addresses
        let server_ip = "0.0.0.0:";
        let server_address_election = format!("{}{}", server_ip, port_election);
        let server_address_leader = format!("{}{}", server_ip, port_leader);
        let server_address_client = format!("{}{}", server_ip, port_client);
        let server_address_server = format!("{}{}", server_ip, port_server);
        let server_address_server_recv = format!("{}{}", server_ip, port_server_recv);
        let server_address_server_leader = format!("{}{}", server_ip, port_server_leader);

        let server_address_bully = format!("{}{}", server_ip, port_bully);

        // Bind server and client sockets asynchronously
        let socket_election = Arc::new(UdpSocket::bind(server_address_election).await.expect("Failed to bind server socket"));
        let socket_leader = Arc::new(UdpSocket::bind(server_address_leader).await.expect("Failed to bind server socket"));
        let socket_client_send = Arc::new(UdpSocket::bind(server_address_client).await.expect("Failed to bind client socket"));
        let socket_server = Arc::new(UdpSocket::bind(server_address_server).await.expect("Failed to bind server socket"));
        let socket_bully = Arc::new(UdpSocket::bind(server_address_bully).await.expect("Failed to bind bully socket"));
        let socket_server_recv = Arc::new(UdpSocket::bind(server_address_server_recv).await.expect("Failed to bind server socket"));
        let socket_server_leader = Arc::new(UdpSocket::bind(server_address_server_leader).await.expect("Failed to bind server socket"));

        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");
        let db = Arc::new(Mutex::new(HashMap::new())); // Initialize db as Arc<Mutex>



        Server {
            id,
            failed: Arc::new(RwLock::new(failed)),
            leader,
            election,
            socket_client_send,
            socket_leader,
            socket_election,
            socket_bully,
            socket_server_leader,
            socket_server,
            socket_server_recv,
            multicast_addr,
            interface_addr,
            port_leader,
            port_election,
            port_bully,
            port_server_recv,
            db,
            random_number: Arc::new(Mutex::new(0)),
            client_address: Arc::new(Mutex::new(Ipv4Addr::from_str("0.0.0.0").expect("Invalid client address"))),
        }
    }

    pub async fn join(self:Arc<Self>) -> std::io::Result<()> {        
        middleware::join(self.socket_server.clone(), self.interface_addr, self.multicast_addr)
            .await
            .expect("Failed to join multicast group");

        middleware::join(self.socket_leader.clone(), self.interface_addr, self.multicast_addr)
        .await
        .expect("Failed to join multicast group");
        Ok(())
    }

    pub async fn send_leader(self:Arc<Self>) -> std::io::Result<()> {
        print!("inside send_leader");
        let socket_leader = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_leader;
        let id = self.id;
    
        // Spawn a new async task for sending leader messages
        // tokio::spawn(async move {
            let message = format!("Lead {}", id.clone());
            // Continue sending messages while not the leader
            
            while self.leader.load(Ordering::SeqCst) && !self.election.load(Ordering::SeqCst) {
                
                println!("Sending message: {}", message);
                match middleware::send_message(socket_leader.clone(), multicast_addr, port_server, message.clone()).await {
                    Ok(_) => println!("Message sent successfully"),
                    Err(e) => eprintln!("Failed to send RPC: {}", e),
                }
                // Sleep for 1 second before sending the next message
                time::sleep(Duration::from_secs(1)).await;
            }

        // });
    
        Ok(())
    }

   
    pub async fn recv_leader(self: Arc<Self>) -> std::io::Result<()> {
        loop {
            
            let mut buf = vec![0u8; 1024];
            let socket_server = self.socket_leader.clone();
            println!("inside recv_leader");
            match socket_server.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    let data = buf[..len].to_vec();
                    let message = String::from_utf8_lossy(&data).to_string();
                    println!("Updated leader status:");
                    println!("LEADER Received '{}' from {}", message, addr);
                    
                    
                        
                        if let Some(id_str) = message.strip_prefix("Lead ") {
                            if let Ok(id) = id_str.trim().parse::<u32>() {
                                if id == self.id {
                                    self.leader.store(true, Ordering::SeqCst);
                                    println!("This server is now the leader.");
                                    self.clone().client_send_leader().await.expect("Failed to send leader");

                                } else {
                                    self.leader.store(false, Ordering::SeqCst);
                                }
                                self.election.store(false, Ordering::SeqCst);
                            }
                        }
                }
                Err(e) => eprintln!("Failed to receive message: {}", e),
                _ => {}
            }
        }
    }

    pub async fn recv_election(self: Arc<Self>) -> std::io::Result<()> {
        let self_clone = Arc::clone(&self);
        loop {
            let mut buf = vec![0u8; 1024];
            let socket_server = self_clone.socket_election.clone();
            let db = self_clone.db.clone();
    
            println!("inside recv_election");
    
            match socket_server.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    let data = buf[..len].to_vec();
                    let message = String::from_utf8_lossy(&data).to_string();
                    println!("ELECTION Received '{}' from {}", message, addr);
    
                    // Parse the message and update the database.
                    if let Some((key, value)) = message.split_once(":") {
                        let key = key.trim();
                        if let Ok(parsed_key) = key.parse::<u32>() {
                            if let Ok(value) = value.trim().parse::<u32>() {
                                {
                                    // Lock the database only for the duration of the update.
                                    let mut db_lock = db.lock();
                                    db_lock.await.insert(parsed_key, value);
                                }
                                println!("Updated database with {}:{}", parsed_key, value);
    
                                // Determine if a new leader should be chosen.
                                let self_clone = Arc::clone(&self_clone);
                                if let Ok(Some(new_leader)) = self_clone.clone().choose_leader().await {
                                    // Access the `id` field from `self` directly
                                    if self_clone.clone().id == new_leader {
                                        println!("This server is now the leader.");
                                        
                                        self_clone.leader.store(true, Ordering::SeqCst);
                                        println!("Sending message: TO CLient");
                                        // send back to the leader the ip address
                                    } else {
                                        self_clone.leader.store(false, Ordering::SeqCst);
                                    }
                                    self.election.store(false, Ordering::SeqCst);
                                }
                            } else {
                                eprintln!("Failed to parse value from message: {}", message);
                            }
                        } else {
                            eprintln!("Failed to parse key from message: {}", message);
                        }
                    } else {
                        println!("ELECTION Received invalid message format from {}", addr);
                    }
                }
                Err(e) => eprintln!("Failed to receive message: {}", e),
            }
        }
    }

    pub async fn choose_leader(self: Arc<Self>) -> std::io::Result<Option<u32>> {
        let db = self.db.lock();
        let mut min_key: Option<u32> = None;
        let mut min_value: Option<u32> = None;

        for (&key, &value) in db.await.iter() {
            if min_value.is_none() || value < min_value.unwrap() {
                min_value = Some(value);
                min_key = Some(key);
            }
        }
        
        Ok(min_key) // Return the key with the lowest value or None if the map is empty
    }
    
    pub async fn client_send_leader(self: Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_client_send.clone();  // Get the socket for sending to clients
        let interface_addr = self.interface_addr;              // Server's interface address
        let port_server_recv = self.port_server_recv;          // Client's receiving port
        let id = self.id;   
        // read client address
        let client_addr = self.client_address.clone();                                   // Server ID
    
        // Check if this server is the leader and the election is over
        let leader = self.leader.load(Ordering::SeqCst);
        let election = self.election.load(Ordering::SeqCst);
    
        let mut message = String::new();
    
        // If this server is the leader and election has finished, construct the message
        if leader && !election {
            message = format!("{}:{}", interface_addr, port_server_recv);
        } else {
            println!("This server is not the leader, not sending any message to the client.");
            return Ok(()); // Exit early if not the leader or election is ongoing
        }
    
        println!("Sending message to client: {}", message);
    
        // Create the destination socket address using the interface address and port
        let client_addr = *client_addr.lock().await;
        let destination_addr = SocketAddr::new(client_addr.into(), 9002);
    
        // Send the message to the client at the destination address
        match socket_server.send_to(message.as_bytes(), &destination_addr).await {
            Ok(_) => {
                println!("Message sent successfully to {}", destination_addr);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to send message to client: {}", e);
                Err(e)
            }
        }
    }
    

    pub async fn send_info(self: Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_election;
        let id = self.id;

        while self.election.load(Ordering::SeqCst) {
            let mut sys = System::new_all();
            sys.refresh_all(); // Refresh system information

            // Get memory and CPU usage
            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let cpu_usage = sys.global_cpu_usage();

            // Update shared CPU and memory usage
            let cpu = cpu_usage as u32;
            let mem = (used_memory as f32 / total_memory as f32 * 100.0) as u32;
            let message = format!("{}:{}", id, cpu * mem);

            println!("Sending message: {}", message);
            middleware::send_message(socket_server.clone(), multicast_addr, port_server, message)
                .await
                .expect("Failed to send RPC");

            time::sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }

    // mainly port_server and port_client
    pub async fn load_env_vars(&self) -> (String, String) {
        
        dotenv().ok();
        let server_port_recv = env::var("SERVER_PORT_RECV").expect("SERVER_PORT_RECV not set");
        let server_port_send = env::var("SERVER_PORT_SEND").expect("SERVER_PORT_SEND not set");

        (server_port_recv, server_port_send)
    }

    pub async fn bind_socket(&self,address: &str) -> Arc<UdpSocket> {
        Arc::new(UdpSocket::bind(address).await.expect("Failed to bind socket"))
    }


    pub fn initialize_image_buffer(&self) -> Arc<Mutex<Vec<u8>>> {
        Arc::new(Mutex::new(Vec::new()))
    }

    pub fn initialize_client_addr(&self) -> Arc<Mutex<Option<std::net::SocketAddr>>> {
        Arc::new(Mutex::new(None))
    }


    pub async fn start_receive_task(
        &self,
        image_data: Arc<Mutex<Vec<u8>>>,
        client_addr: Arc<Mutex<Option<std::net::SocketAddr>>>,
    ) {
        
        let socket_server = self.socket_server_recv.clone();
        let socket_client = self.socket_client_send.clone();
        let socket_server_leader = self.socket_server_leader.clone();
    
        tokio::spawn(async move {
            receive_image_data(socket_server, socket_client, image_data, client_addr)
                .await
                .expect("Failed to receive image data");
        });
    }

    pub async fn wait_for_shutdown(&self) -> io::Result<()> {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Shutdown signal received.");
        Ok(())
    }
    //recieve "START" message from client in loop and check it
    pub async fn recieve_start_elections(self:Arc<Self>) -> std::io::Result<()> {
        let socket_server = self.socket_server_leader.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_election;
        let id = self.id;
        let self_clone = Arc::clone(&self);
        println!("WAITING FOR START MESSAGE");
        loop {
            let mut buf = vec![0u8; 1024];
            let self_clone = Arc::clone(&self_clone); 
            match socket_server.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    
                    let data = buf[..len].to_vec();
                    let message = String::from_utf8_lossy(&data).to_string();
                    println!("******************************************************************************************************************Received '{}' from {}", message, addr);
                    let mut client_address_lock = self.client_address.lock().await;
                    *client_address_lock = match addr.ip() {
                        std::net::IpAddr::V4(ipv4) => ipv4,
                        std::net::IpAddr::V6(_) => panic!("Expected an IPv4 address"),
                    };

                    if message == "START" {
                        self_clone.start_election().await.expect("Failed to start elections");
                    }
                }
                Err(e) => eprintln!("Failed to receive message: {}", e),
            }
        }
    }

    pub async fn start_election(self: Arc<Self>) -> std::io::Result<()> {

        self.election.store(true, Ordering::SeqCst);
        let elections = self.election.load(Ordering::SeqCst);
        println!("Starting election... {} ", elections);
       

        Ok(())
    }
    
}


// encoding and decoding 
pub fn server_encode_image(input_image: &str, encoded_image: &str, cover_image: &str) {
    image_processor::encode_image(input_image.to_string(), encoded_image.to_string(), cover_image.to_string());
}
pub fn server_decode_image(encoded_image: &str, output_image: &str) {
    image_processor::decode_image(encoded_image.to_string(), output_image.to_string());
}

pub async fn send_encoded_image(
    socket: Arc<UdpSocket>,
    client_addr: Arc<Mutex<Option<std::net::SocketAddr>>>,
    image_path: &str,
) -> io::Result<()> {
    let mut file = File::open(image_path)?;
    let file_size = file.metadata()?.len() as f64;
    let mut buf = vec![0u8; 1024];
    let client_addr = client_addr.lock().await.unwrap(); // Dynamic client address
    let mut index: i32 = 0;
    let mut bytes_sent = 0.0;

    println!("Sending encoded image to client...");

    while let Ok(bytes_read) = file.read(&mut buf) {
        if bytes_read == 0 {
            break;
        }

        let mut chunk = vec![0; 4 + bytes_read];
        chunk[..4].copy_from_slice(&index.to_be_bytes());
        chunk[4..].copy_from_slice(&buf[..bytes_read]);

        // Retry loop for sending each chunk
        let mut retries = 0;
        const MAX_RETRIES: usize = 5;

        loop {
            socket.send_to(&chunk, client_addr).await?;
            let mut ack_buf = [0u8; 4];
            match timeout(Duration::from_secs(2), socket.recv_from(&mut ack_buf)).await {
                Ok(Ok((4, _))) if ack_buf == index.to_be_bytes() => {
                    index += 1;
                    bytes_sent += bytes_read as f64;
                    print!("\rProgress: {:.2}% - Chunk {} acknowledged", bytes_sent / file_size * 100.0, index);
                    break;
                }
                _ => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        println!("\nFailed after {} retries. Skipping chunk {}", MAX_RETRIES, index);
                        break;
                    }
                    println!("\nRetrying chunk {}... (Attempt {}/{})", index, retries, MAX_RETRIES);
                }
            }
        }
    }

    socket.send_to(b"END", client_addr).await?; // Final END message to the actual client
    println!("\nEncoded image sent to client.");
    Ok(())
}


async fn receive_image_data(
    server_socket_recv: Arc<UdpSocket>,
    server_socket_send: Arc<UdpSocket>,
    image_data: Arc<Mutex<Vec<u8>>>,
    client_addr_sending: Arc<Mutex<Option<std::net::SocketAddr>>>,
) -> io::Result<()> {
    loop {
        let mut buf = vec![0u8; 1028]; // 4 bytes for index + 1024 bytes for data
        let mut expected_chunk_index = 0;
        let mut total_chunks_received = 0;

        println!("\n\n**************************************************");
        println!("**************************************************");
        println!("Waiting to receive new image...");

        // Image file paths
        let server_received_image_name = "server_received_image.png";
        let server_encoded_image_name = "server_encoded_image.png";
        let mask_image_name = "mask2.jpg";
        let mask_image_path = "./masks/";

        loop {
            // Receive a packet from the client
            let (len, mut addr) = server_socket_recv.recv_from(&mut buf).await.expect("Failed to receive data");

            // Create a new `SocketAddr` with the updated port (addr.port() + 2)
            let new_addr = SocketAddr::new(addr.ip(), addr.port() + 1);
            // println!("---- Client address for sending: {:?}", new_addr);

            // Store modified client address for response
            *client_addr_sending.lock().await = Some(new_addr);

            if len == 3 && &buf[..len] == b"END" {
                println!("\nEnd of transmission received. Saving and encoding file...");

                // Write accumulated image data to a file
                let mut file = File::create(server_received_image_name)?;
                file.write_all(&image_data.lock().await)?;

                // Encode the image
                server_encode_image(
                    server_received_image_name,
                    server_encoded_image_name,
                    &format!("{}{}", mask_image_path, mask_image_name)
                );

                // Log parameters for debugging before sending the encoded image
                // println!("***************************************************");
                // println!("Server socket send: {:?}", server_socket_send);
                // println!("Client address for sending: {:?}", client_addr_sending);
                // println!("Server encoded image name: {:?}", server_encoded_image_name);

                // Send the encoded image to `new_addr` using `server_socket_send`
                send_encoded_image(server_socket_send.clone(), client_addr_sending.clone(), server_encoded_image_name).await?;
                println!("\nEncoded image sent to client.");

                // Clear image data for the next transmission
                *image_data.lock().await = Vec::new(); // Clear the buffer
                *client_addr_sending.lock().await = None; // Reset client address for new connection
                break;

            } else {
                let chunk_index = u32::from_be_bytes(buf[..4].try_into().unwrap());

                // Only process the expected chunk index
                if chunk_index == expected_chunk_index {
                    let mut data = image_data.lock();
                    data.await.extend_from_slice(&buf[4..len]);
                    // print!("\rReceived chunk {}, expected chunk index: {}", chunk_index, expected_chunk_index);
                    total_chunks_received += 1;

                    // Send acknowledgment back to the client using the original port
                    server_socket_recv.send_to(&chunk_index.to_be_bytes(), addr).await?;
                    expected_chunk_index += 1;
                } else {
                    println!(
                        "Out-of-order chunk received (expected {}, got {}). Ignoring.",
                        expected_chunk_index, chunk_index
                    );
                }
            }
        }

        // Log the total chunks received
        println!("\nTotal chunks received from client: {}", total_chunks_received);
        println!("**************************************************");
        println!("**************************************************\n\n");
    }
}


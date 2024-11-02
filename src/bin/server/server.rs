use tokio::net::UdpSocket;
use std::fmt::format;
use std::net::Ipv4Addr;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use tokio::time::{self, Duration};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex; // Use tokio's async Mutex
use tokio::task;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use tokio::sync::broadcast;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use tokio::time::{timeout};
use sysinfo::{System};
use crate::image_processor;
use tokio::signal;
use std::io::{self, Read, Write};
use std::fs::File;
use crate::middleware;

pub struct Server {
    id: u32,
    failed: Arc<RwLock<bool>>,
    leader: Arc<AtomicBool>,     // Changed from RwLock to AtomicBool
    election: Arc<AtomicBool>,   // Changed from RwLock to AtomicBool
    socket_client: Arc<UdpSocket>, // for client to server communication
    socket_leader: Arc<UdpSocket>, // for server to server communication
    socket_election: Arc<UdpSocket>, // for server to server communication
    socket_bully: Arc<UdpSocket>, // for bully algorithm
    socket_server:  Arc<UdpSocket>, // for bully algorithm
    multicast_addr: Ipv4Addr,
    interface_addr: Ipv4Addr,
    port_leader: u16,
    port_election: u16,
    port_bully: u16,
    db: Arc<Mutex<HashMap<u32, u32>>>, // Wrap db in Arc<Mutex> for thread-safe access
    random_number: Arc<Mutex<u32>>,
}







impl Server {
    pub async fn new(id: u32, failed: bool, leader: Arc<AtomicBool>, election: Arc<AtomicBool>) -> Server {
        dotenv().ok();
        
        let port_leader = env::var("PORT_LEADER").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_election = env::var("PORT_ELECTION").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_client = env::var("PORT_CLIENT").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");
        let port_bully = env::var("PORT_BULLY").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
        let port_server = env::var("PORT_SERVER").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");

        // Define server addresses
        let server_ip = "0.0.0.0:";
        let server_address_election = format!("{}{}", server_ip, port_election);
        let server_address_leader = format!("{}{}", server_ip, port_leader);
        let server_address_client = format!("{}{}", server_ip, port_client);
        let server_address_server = format!("{}{}", server_ip, port_server);
        
        let server_address_bully = format!("{}{}", server_ip, port_bully);

        // Bind server and client sockets asynchronously
        let socket_election = Arc::new(UdpSocket::bind(server_address_election).await.expect("Failed to bind server socket"));
        let socket_leader = Arc::new(UdpSocket::bind(server_address_leader).await.expect("Failed to bind server socket"));
        let socket_client = Arc::new(UdpSocket::bind(server_address_client).await.expect("Failed to bind client socket"));
        let socket_server = Arc::new(UdpSocket::bind(server_address_server).await.expect("Failed to bind server socket"));
        let socket_bully = Arc::new(UdpSocket::bind(server_address_bully).await.expect("Failed to bind bully socket"));


        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");
        let db = Arc::new(Mutex::new(HashMap::new())); // Initialize db as Arc<Mutex>



        Server {
            id,
            failed: Arc::new(RwLock::new(failed)),
            leader,
            election,
            socket_client,
            socket_leader,
            socket_election,
            socket_bully,
            socket_server,
            multicast_addr,
            interface_addr,
            port_leader,
            port_election,
            port_bully,
            db,
            random_number: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn join(self:Arc<Self>) -> std::io::Result<()> {        
        middleware::join(self.socket_election.clone(), self.interface_addr, self.multicast_addr)
            .await
            .expect("Failed to join multicast group");

        middleware::join(self.socket_leader.clone(), self.interface_addr, self.multicast_addr)
        .await
        .expect("Failed to join multicast group");
        Ok(())
    }

    
    pub async fn start_election(self: Arc<Self>) -> std::io::Result<()> {

        // Spawn each task to run concurrently
        self.election.store(true, Ordering::SeqCst);
        println!("Starting election...");
        // let send_info_handle = task::spawn(self.clone().send_info());
        // let recv_election_handle = task::spawn(self.clone().recv_election());

        // // Wait for all tasks to finish concurrently
        // let results = tokio::join!(
        //     send_info_handle,
        //     recv_election_handle
        // );

        // // Check for errors in any of the tasks
        // if let Err(e) = results.0 {
        //     eprintln!("Error in send_leader: {:?}", e);
        // }
        // if let Err(e) = results.1 {
        //     eprintln!("Error in send_info: {:?}", e);
        // }
        // // if let Err(e) = results.2 {
        // //     eprintln!("Error in recv_election: {:?}", e);
        // // }
        // // if let Err(e) = results.3 {
        // //     eprintln!("Error in recv_leader: {:?}", e);
        // // }

        Ok(())
    }

    pub async fn send_leader(self:Arc<Self>) -> std::io::Result<()> {

        let socket_leader = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_leader;
        let id = self.id;
    
        // Spawn a new async task for sending leader messages
        // tokio::spawn(async move {
            let message = format!("Lead {}", id.clone());
            println!("Yehia is gay");
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
    pub async fn send_leader_only(self:Arc<Self>, id: u32) -> std::io::Result<()> {
        let socket_leader = self.socket_server.clone();
        let multicast_addr = self.multicast_addr;
        let port_server = self.port_leader;

    
        // Spawn a new async task for sending leader messages
        tokio::spawn(async move {
            let message = format!("Lead {}", id.clone());
            
        // Continue sending messages while not the leader
        
            println!("Sending message: {}", message);
            match middleware::send_message(socket_leader.clone(), multicast_addr, port_server, message.clone()).await {
                Ok(_) => println!("Message sent successfully"),
                Err(e) => eprintln!("Failed to send RPC: {}", e),
            }

            // Sleep for 1 second before sending the next message
            
        });
    
        Ok(())
    }
    
    pub async fn recv_leader(self: Arc<Self>) -> std::io::Result<()> {
        loop {
            
            let mut buf = vec![0u8; 1024];
            let socket_server = self.socket_leader.clone();
            
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
                                } else {
                                    self.leader.store(false, Ordering::SeqCst);
                                }
                                self.election.store(false, Ordering::SeqCst);
                            }
                        }
                }
                Err(e) => eprintln!("Failed to receive message: {}", e),
            }
        }
    }
    
    pub async fn recv_election(self: Arc<Self>) -> std::io::Result<()> {
        loop {
            let mut buf = vec![0u8; 1024];
            let socket_server = self.socket_election.clone();
            let db = self.db.clone();
    
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
                                let self_clone = Arc::clone(&self);  // Clone once for the async block

                                if let Ok(Some(new_leader)) = self_clone.choose_leader() {
                                    // Access the `id` field from `self_clone` directly
                                    if self.id == new_leader {
                                        println!("This server is now the leader.");
                                        self.leader.store(true, Ordering::SeqCst);
                                    } else {
                                        self.leader.store(false, Ordering::SeqCst);
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
    
    
    
    
    

    pub fn choose_leader(self: Arc<Self>) -> std::io::Result<Option<u32>> {
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

    //algorithm takes in a port number, an id you specify (mostly for debugging), and an array of strings which is the other hosts' ips


    pub async fn send_bully_info(self:Arc<Self>){
        let multicast_address = SocketAddr::new(self.multicast_addr.into(), self.port_bully); //multicast address + bully port dk if this is right

        let election_socket = self.socket_bully.clone();
        task::spawn(async move {
            let random_number = *self.random_number.lock().unwrap();
            let msg = (self.id, random_number, *self.failed.read().unwrap());
            let msg_bytes = bincode::serialize(&msg).unwrap();
    
       
            if let Err(e) = election_socket.send_to(&msg_bytes, multicast_address).await {
                eprintln!("Failed to send multicast message: {}", e);
            }
        });
    }

    pub async fn bully_algorithm(self: Arc<Self>, info: (i32, u32, bool)) {
        let socket = self.socket_bully.clone();
        let multicast_addr = self.multicast_addr;
        let port_bully = self.port_bully;
    
        println!("Host {} listening on {}", self.id, port_bully);
    
        let (tx, mut rx): (broadcast::Sender<(u32, u32, bool)>, broadcast::Receiver<(u32, u32, bool)>) = broadcast::channel(10);  //this is a mpsc channel not a network broadcast channel
        let listening_tx: broadcast::Sender<(u32, u32, bool)> = tx.clone();
        let socket_listener = socket.clone();
    
     
        task::spawn(async move { //task for listening for messages
            let mut buf = [0u8; 1024]; 
            while let Ok((size, addr)) = socket_listener.recv_from(&mut buf).await {
                if let Ok((id, random_number, failed)) = bincode::deserialize(&buf[..size]) {
                    println!("Received message from {}: (id: {}, random_number: {}, failed: {})", addr, id, random_number, failed);
                    let _ = listening_tx.send((id, random_number, failed)); //store in the tx channel
                }
            }
        });
    
        
        let mut random_number = self.random_number.lock().unwrap();
        *random_number = rand::thread_rng().gen_range(0..100); //generate the number for sim fail
        println!("Host {} generated identifier: {:?}", self.id, self.random_number);
    
        
        let election_socket = socket.clone();
        let multicast_address = SocketAddr::new(multicast_addr.into(), port_bully); //multicast address + bully port dk if this is right
    
        self.clone().send_bully_info().await;
    
        let mut numbers = vec![(self.id, self.random_number.clone())];
        let mut received_hosts = HashSet::new(); //for o(1) lookup on whether we have received a message from a host
        received_hosts.insert(self.id);
        received_hosts.insert(info.0 as u32);
        numbers.push((info.0 as u32, Arc::new(Mutex::new(info.1))));
    
        let start_time = time::Instant::now();
        let timeout_duration = Duration::from_secs(5); //dk if 5 is too much but probably not since we're simulating failure anyway
    
        // Main loop for receiving messages within the timeout
        while start_time.elapsed() < timeout_duration {
            match time::timeout(timeout_duration - start_time.elapsed(), rx.recv()).await {
                Ok(Ok((id, random_number, failed))) => {
                    if !received_hosts.contains(&id) {
                        numbers.push((id, self.random_number.clone()));
                        received_hosts.insert(id);
                    }
                }
                _ => break, // Break on timeout or error
            }
        }
    
        if let Some(max_host) = numbers.iter().max_by_key(|x| *x.1.lock().unwrap()) { //check if we're the highest number
            let mut failed = self.failed.write().unwrap();
            *failed = max_host.0 == self.id;
            let failed = *self.failed.read().unwrap();
            println!("Host {}: {}", self.id, if failed { "failed" } else { "active" });
        }
    }
    
    


    pub async fn bully_listener(self: Arc<Self>) {
        let socket = self.socket_bully.clone();
        let multicast_addr = self.multicast_addr;
        let port_bully = self.port_bully;

        let mut buf = [0u8; 1024];

        loop {
            let (size, _) = socket.recv_from(&mut buf).await.unwrap();
            if let Ok((id, random_number, failed)) = bincode::deserialize(&buf[..size]) {
                println!("Starting sim fail due to received message: {:?}", (id, random_number, failed));
                self.clone().bully_algorithm((id, random_number, failed)).await; 
            }
        }
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
        
        let socket_server = self.socket_server.clone();
        let socket_client = self.socket_client.clone();
    
        tokio::spawn(async move {
            Self::receive_image_data(socket_server, socket_client, image_data, client_addr)
                .await
                .expect("Failed to receive image data");
        });
    }

    pub async fn wait_for_shutdown(&self) -> io::Result<()> {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Shutdown signal received.");
        Ok(())
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
pub async fn receive_image_data(
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
                Self::server_encode_image(
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


}




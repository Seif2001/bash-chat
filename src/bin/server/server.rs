use tokio::net::UdpSocket;
use std::fmt::format;
use std::net::Ipv4Addr;
use dotenv::dotenv;
use std::env;
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

// #[tokio::main]
// async fn main() -> io::Result<()> {
//     // let (port_server, port_client) = load_env_vars().await;
//     // let (server_port_recv, server_port_send) = load_env_vars().await;
//     let server_port_recv = "6372";
//     let server_port_send = "6373";

//     let server_ip = "127.0.0.1:".to_string();
//     let server_address_recv = format!("{}{}", server_ip, server_port_recv);
//     // 127.0.0.1:6372
//     let server_address_send = format!("{}{}", server_ip, server_port_send);
//     // 127.0.0.1:6373

//     let _server_address_recv = bind_socket(&server_address_recv).await;
//     let _server_address_send = bind_socket(&server_address_send).await;

//     println!("Listening for UDP packets on {}", server_address_recv);

//     let image_data = initialize_image_buffer();
//     let client_addr = initialize_client_addr();
    
//     // middle ware function
//     start_receive_task(_server_address_recv.clone(), _server_address_send.clone(), image_data.clone(), client_addr.clone()).await;
//     wait_for_shutdown().await
// }


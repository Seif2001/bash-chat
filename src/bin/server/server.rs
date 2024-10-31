use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::Ipv4Addr;
use dotenv::dotenv;
use std::env;
use std::str::FromStr;

mod middleware;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Environment variables for ports and IP addresses
    let port_server = env::var("PORT_SERVER").expect("PORT_SERVER not set");
    let port_client = env::var("PORT_CLIENT").expect("PORT_CLIENT not set");
    let port_multicast = env::var("PORT_MULTICAST").expect("PORT_MULTICAST not set");

    let server_ip = String::from("127.0.0.1:");

    // Full server addresses for different roles
    let server_address_server = format!("{}{}", server_ip, &port_server);
    let server_address_client = format!("{}{}", server_ip, &port_client);
    let server_address_multicast = format!("{}{}", server_ip, &port_multicast);

    
    // Parse ports and bind sockets
    let port_server = port_server.parse::<u16>().expect("Invalid server port");
    let socket_multicast = Arc::new(UdpSocket::bind(server_address_multicast).await.expect("Failed to bind multicast socket"));
    let socket_server = Arc::new(UdpSocket::bind(server_address_server).await.expect("Failed to bind server socket"));
    let socket_client = Arc::new(UdpSocket::bind(server_address_client).await.expect("Failed to bind client socket"));
    
    println!("Listening for UDP packets on 127.0.0.1:{}", port_client);
    // Multicast and interface addresses from env variables
    let multicast_address_env = env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set");
    let interface_address_env = env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set");

    let interface_addr = Ipv4Addr::from_str(&interface_address_env).expect("Invalid interface address");
    let multicast_addr = Ipv4Addr::from_str(&multicast_address_env).expect("Invalid multicast address");
    
    let db = Arc::new(Mutex::new(HashMap::new()));

    // Join multicast group
    middleware::join(socket_multicast.clone(), interface_addr, multicast_addr).await.expect("Failed to join multicast group");

    // Set up RPC send and receive tasks
    middleware::send_rpc(socket_server.clone(), multicast_addr, port_server).await.expect("Failed to send RPC");

    middleware::recv_rpc(socket_server.clone()).await.expect("Failed to receive RPC");

    // Main loop for processing incoming messages
    loop {
        let db = db.clone();
        let socket_client = socket_client.clone();
        let mut buf = vec![0u8; 1024];
        let (len, addr) = socket_client.recv_from(&mut buf).await.expect("Failed to receive from client socket");
        let data = buf[..len].to_vec();

        tokio::spawn(async move {
            middleware::process(socket_client, db, data, addr).await;
        });
    }
}

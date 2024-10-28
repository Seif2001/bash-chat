use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::{Ipv4Addr};


use dotenv::dotenv;
use std::env;

mod middleware;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Bind a UDP socket to port 6379
    let port = env::var("PORT").expect("MULTICAST_ADDRESS not set");
    let server_ip = String::from("0.0.0.0:");
    let server_address = server_ip+&port;

    let socket = Arc::new(UdpSocket::bind(server_address).await.unwrap());
    
    println!("Listening for UDP packets on 127.0.0.1:{}", port);

    let multicast_address_env = env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set");
    let interface_address_env = env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set");

    let interface_addrs: Vec<&str> = interface_address_env.split(".").collect();
    let multicast_addrs: Vec<&str> = multicast_address_env.split(".").collect();


    let interface_addr = Ipv4Addr::new(interface_addrs[0].parse().unwrap(), interface_addrs[1].parse().unwrap(), interface_addrs[2].parse().unwrap(), interface_addrs[3].parse().unwrap());
    let multicast_addr = Ipv4Addr::new(multicast_addrs[0].parse().unwrap(), multicast_addrs[1].parse().unwrap(), multicast_addrs[2].parse().unwrap(), multicast_addrs[3].parse().unwrap());
    
    let db = Arc::new(Mutex::new(HashMap::new()));


    middleware::join(socket.clone(), interface_addr, multicast_addr).await.unwrap();

    loop {
       
        let db = db.clone();
        let socket = socket.clone();
        let mut buf = vec![0u8; 1024];
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
        let data = buf[..len].to_vec();
        tokio::spawn(async move {
            middleware::process(socket, db, data, addr).await;

        });
    }
}


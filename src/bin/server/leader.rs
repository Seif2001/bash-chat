use mini_redis::{client, server};
use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::task::Id;
use std::sync::Arc;
use std::collections::HashMap;
use std::net::Ipv4Addr;

use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;



pub async fn send_leader(servers: Arc<Mutex<HashMap<u32, Node>>>, socket: &Socket, leader_id: u32, config: &Config) {
    // Lock the servers Mutex to access the HashMap
    let db = servers.lock().await;

    // Check if the leader_id exists in the HashMap
    if let Some(node) = db.get(&leader_id) {
        let term = node.term;  // Safely access the term of the leader

        let message = format!("{} : {}", term, leader_id);
        println!("Sending leader message: {}", message);

        let dest = (config.multicast_addr, config.port_election_rx);    

        // Use the instance of Socket passed as a reference
        let socket_election = socket.socket_election_tx.clone();
        com::send(&socket_election, message, dest).await.expect("Failed to send leader message");
    } else {
        println!("Leader ID {} not found in the servers map.", leader_id);
    }
}

pub async fn recv_leader(socket: &Socket, servers: Arc<Mutex<HashMap<u32, Node>>>) {
    println!("Waiting for leader message");
    let socket_election_rx = socket.socket_election_rx.clone();
    tokio::spawn(async move {
        loop {
            let (message, _) = com::recv(&socket_election_rx).await.expect("Failed to receive message");
            let message = message.trim();
            if let Some((term_str, leader_id_str)) = message.split_once(" : ") {
                // Parse the term and leader_id from the message
                let term = term_str.parse::<u32>().expect("Invalid term");
                let leader_id = leader_id_str.parse::<u32>().expect("Invalid leader id");
                println!("Received leader message: {} : {}", term, leader_id);
                set_leader(leader_id, servers.clone(), term).await;
            }
        }
    });
}


#[derive(Debug)]
pub struct Node {
    pub is_leader: bool,
    pub is_dos_leader:bool,
    pub is_failed: bool,
    pub current_leader: u32,
    pub current_dos_leader:u32,
    pub term: u32,
}

impl Node {
    pub fn new(is_leader: bool, is_dos_leader:bool,is_failed: bool, current_leader: u32,current_dos_leader:u32) -> Self {
        Node {
            is_leader,
            is_dos_leader,
            is_failed,
            current_leader,
            current_dos_leader,
            term: 0,
        }
    }

    
}

pub async fn set_leader(leader_id: u32, servers: Arc<Mutex<HashMap<u32, Node>>>, term: u32) {
    
    // change the leader of the other nodes
    let mut db = servers.lock().await;
    for (key, node) in db.iter_mut() {
        if *key == leader_id {
            node.is_leader = true;
        } else {
            node.is_leader = false;
        }
        node.current_leader = leader_id;
        node.term = term;
    }
    println!("Updated servers map: {:?}", db);
    drop(db);
}



pub async fn elect_leader(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id:u32, socket: &Socket, config: &Config, term: u32, client_addr: Ipv4Addr)->std::io::Result<()>{
    let mut leader_id: u32 = my_id;
    let numbers_of_servers:u32 = servers.lock().await.len().try_into().unwrap();
    let db = servers.lock().await;

    for (key, node) in db.iter(){
        if node.is_leader {
            leader_id = *key;
        }
    }
    println!("current leader is {}", leader_id);
    drop(db); // Unlock the mutex before locking it again

    while let Some(node) = servers.lock().await.get_mut(&leader_id) {
        leader_id = (leader_id+1) % numbers_of_servers;  // Increment the leader_id to check the next server
        println!("Checking leader at id {}", leader_id);
        if !node.is_failed {
            println!("Found a leader at id {}: {:?}", leader_id, node);
            break;
        }
    }

    if leader_id == my_id{
        println!("I am the leader");
        send_leader(servers, socket, leader_id, config).await;
        let socket = socket.clone();
        send_leader_to_client(socket, client_addr, config.port_client_tx_leader, leader_id).await;

    }
    Ok(())
    
}


pub async fn send_leader_to_client(socket:&Socket, client_addr: Ipv4Addr, port: u16, leader_id: u32){
    let message = format!("{}:{}", "LEADER", leader_id);
    let dest = (client_addr, port);
    let socket = socket.socket_client_leader_tx.clone();
    println!("Sending to clien at {}:{}", dest.0, dest.1);
    com::send(&socket,message, dest).await.expect("Failed to send leader to client");
}

pub async fn elections(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id: u32, socket: &Arc<Socket>, config: &Arc<Config>) {
    // Start a task to receive leader messages
    let socket_clone = Arc::clone(socket);
    let servers_clone = Arc::clone(&servers);
    tokio::spawn(async move {
        recv_leader(&socket_clone, servers).await;
    });

    // check if received election from client
    let mut term = 0;
    let socket_client = socket.socket_client_elections_rx.clone();
    let servers = Arc::clone(&servers_clone);
    let socket = Arc::clone(&socket);
    let config = Arc::clone(config);
    tokio::spawn(async move {
        loop {
            println!("Waiting for election message from client");
            let (message, client_addr) = com::recv(&socket_client).await.expect("Failed to receive message");
            let message = message.trim();
            let client_addr = match client_addr.ip() {
                std::net::IpAddr::V4(addr) => addr,
                _ => panic!("Expected an IPv4 address"),
            };
            if message == "START" {
                println!("Received election message from client");
                let servers_clone = Arc::clone(&servers);
                let socket_clone = Arc::clone(&socket);
                let config_clone = Arc::clone(&config);
                elect_leader(servers_clone, my_id, &socket_clone, &config_clone, term, client_addr).await.expect("Failed to elect leader");
                term += 1;
            }   
        }
    });
}



////// DOS elections //////

pub async fn elections_dos(servers: Arc<Mutex<HashMap<u32, Node>>>,socket: &Arc<Socket>) {
    // Start a task to receive leader messages
    let socket_clone = Arc::clone(socket);
    tokio::spawn(async move {
        recv_leader_dos(&socket_clone, servers).await;
    });
}

pub async fn set_leader_dos(leader_id: u32, servers: Arc<Mutex<HashMap<u32, Node>>>) {
    let mut db = servers.lock().await;
    for (key, node) in db.iter_mut() {
        if *key == leader_id {
            node.is_dos_leader = true;
        } else {
            node.is_dos_leader = false;
        }
        node.current_dos_leader = leader_id;
    }
    println!("Updated servers map: {:?}", db);
    drop(db);
}



pub async fn elect_leader_dos(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id:u32, socket: &Socket, config: &Config)->std::io::Result<()>{
    let mut leader_id: u32 = my_id;
    let db = servers.lock().await;

    for (key, node) in db.iter(){
        if node.is_dos_leader {
            leader_id = *key;
        }
    }
    drop(db); // Unlock the mutex before locking it again

    while let Some(node) = servers.lock().await.get_mut(&leader_id) {
        leader_id = (leader_id+1) % 3;  // Increment the leader_id to check the next server
        println!("Checking leader at id {}", leader_id);
        if !node.is_failed {
            println!("Found a leader at id {}: {:?}", leader_id, node);
            break;
        }
    }

    if leader_id == my_id{
        send_leader_dos(servers, socket, leader_id, config).await;
    }
    Ok(())
    
}


pub async fn send_leader_dos(servers: Arc<Mutex<HashMap<u32, Node>>>, socket: &Socket, leader_id: u32, config: &Config) {
    let db = servers.lock().await;

    if let Some(node) = db.get(&leader_id) {
        let message = format!("{} : {}", 0, leader_id);
        println!("Sending leader message: {}", message);

        let dest = (config.multicast_addr, config.port_dos_election_rx);    

        let socket_election = socket.socket_dos_election_tx.clone();
        com::send(&socket_election, message, dest).await.expect("Failed to send leader message");
    } else {
        println!("Leader ID {} not found in the servers map.", leader_id);
    }
}

pub async fn recv_leader_dos(socket: &Socket, servers: Arc<Mutex<HashMap<u32, Node>>>) {
    println!("Waiting for dos leader message");
    let socket_election_rx = socket.socket_dos_election_rx.clone();
    tokio::spawn(async move {
        loop {
            let (message, _) = com::recv(&socket_election_rx).await.expect("Failed to receive message");
            let message = message.trim();
            if let Some((term_str, leader_id_str)) = message.split_once(" : ") {
                // Parse the term and leader_id from the message
                let term = term_str.parse::<u32>().expect("Invalid term");
                let leader_id = leader_id_str.parse::<u32>().expect("Invalid leader id");
                println!("Received dos leader message: {} : {}", term, leader_id);
                set_leader_dos(leader_id, servers.clone()).await;
            }
        }
    });
}






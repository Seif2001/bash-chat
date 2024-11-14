use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use std::sync::Arc;
use std::collections::HashMap;

use crate::config::{self, Config};
use crate::socket::Socket;
use crate::com;



pub async fn send_leader(socket: &Socket, leader_id: u32, config: &Config) {
    let message = format!("lead {}", leader_id);
    println!("Sending leader message: {}", message);
    let dest = (config.multicast_addr, config.port_election_rx);    

    // Use the instance of Socket passed as a reference
    let socket_election = socket.socket_election_tx.clone();
    com::send(&socket_election, message, dest).await.expect("Failed to send leader message");
}

pub async fn recv_leader(socket: &Socket) {
    let socket_election_rx = socket.socket_election_rx.clone();
    tokio::spawn(async move {
        loop {
            let (message, _) = com::recv(&socket_election_rx).await.expect("Failed to receive message");
            let message = message.trim();
            if message.starts_with("lead") {
                let leader_id = message.split_whitespace().nth(1).expect("Invalid message").parse::<u32>().expect("Invalid leader id");
                println!("Received leader message from id {}: {}", leader_id, message);
            }
        }
    });
}


#[derive(Debug)]
pub struct Node {
    pub is_leader: bool,
    pub is_failed: bool
}



pub async fn elect_leader(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id:u32, socket: &Socket, config: &Config)->std::io::Result<()>{
    let mut leader_id: u32 = my_id;
    let db = servers.lock().await;

    for (key, node) in db.iter(){
        if node.is_leader {
            leader_id = *key;
        }
    }
    drop(db); // Unlock the mutex before locking it again

    while let Some(node) = servers.lock().await.get_mut(&leader_id) {
        leader_id = (leader_id+1) % 3;  // Increment the leader_id to check the next server
        if !node.is_failed {
            println!("Found a leader at id {}: {:?}", leader_id, node);
            break;
        }
    }

    if leader_id == my_id{
        send_leader(socket, leader_id, config).await;
    }
    Ok(())
    
}




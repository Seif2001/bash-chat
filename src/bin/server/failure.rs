use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::time::{self, Duration,timeout};
use tokio::sync::broadcast;
use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use crate::leader::{elect_leader_dos, Node};
use rand::Rng;
use crate::config::{self, Config};
use crate::socket::Socket;
use crate::com;


pub async fn send_random(socket: &Socket, my_id: u32, config: &Config, random_number: u32) {
    let message = bincode::serialize(&(my_id, random_number))
        .expect("Failed to serialize message");

    println!("Sending random message with ID: {} and Random Number: {}", my_id, random_number);

    let dest = (config.multicast_addr, config.port_failover_tx);
    let socket_election = socket.socket_failover_tx.clone();

    com::send_vec(&socket_election, message, dest).await.expect("Failed to send leader message");
}
pub async fn bully_algorithm(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id: u32, socket_a: &Socket, config: &Config, info: (i32, u32)) {

    let socket = socket_a.clone();
    let start_time = time::Instant::now();
    let timeout_duration = Duration::from_secs(5);

    println!("Host {} initializing bully algorithm.", my_id);

    let (tx, mut rx): (broadcast::Sender<(u32, u32)>, broadcast::Receiver<(u32, u32)>) = 
        broadcast::channel(10); // Local in-memory message passing
    let listening_tx = tx.clone();
    let socket_listener = Arc::clone(&socket.socket_failover_tx);

    // Spawn listener task
    tokio::task::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            let result = {
                let listener = socket_listener.lock().await; // Lock the UdpSocket
                timeout(timeout_duration, listener.recv_from(&mut buf)).await
            };

            match result {
                Ok(Ok((size, addr))) => {
                    if let Ok((id, random_number)) = bincode::deserialize::<(u32, u32)>(&buf[..size]) {
                        println!("Received message from {}: (id: {}, random_number: {})", addr, id, random_number);
                        let _ = listening_tx.send((id, random_number)); // Broadcast message
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Error receiving message: {:?}", e);
                }
                Err(_) => {
                    println!("Timeout reached with no messages received within {:?}", timeout_duration);
                    break; 
                }
            }
        }
    });

    let random_number = rand::thread_rng().gen_range(0..100);
    println!("Host {} generated random identifier: {}", my_id, random_number);
    send_random(&socket, my_id, &config, random_number).await;

    let mut numbers = vec![(my_id, random_number)];
    let mut received_hosts = HashSet::new();
    received_hosts.insert(my_id);
    received_hosts.insert(info.0 as u32);

    numbers.push((info.0 as u32, info.1));

    while start_time.elapsed() < timeout_duration {
        match time::timeout(timeout_duration - start_time.elapsed(), rx.recv()).await {
            Ok(Ok((id, random_number))) => {
                if !received_hosts.contains(&id) {
                    numbers.push((id, random_number));
                    received_hosts.insert(id);
                }
            }
            _ => break, // Break on timeout or error
        }
    }
    if numbers.len() == 1 {        
        let mut servers_clone = servers.lock().await;
        for node in servers_clone.values_mut() {
            node.is_failed = false;
        }
        return;
    }

    let max_host = numbers.into_iter().max_by_key(|(_, number)| *number);

    if let Some((max_id, _)) = max_host {
        if max_id == my_id {
            println!("Host {}: failed", my_id);
        } else {
            println!("Host {}: active", my_id);
        }

        let mut servers_clone = servers.lock().await;

        for node in servers_clone.values_mut() {
            node.is_failed = false;
        }

        if let Some(node) = servers_clone.get_mut(&max_id) {
            if max_id == my_id && node.is_dos_leader {
                let _ = elect_leader_dos(servers.clone(), my_id, &socket, config).await; // Pass the Arc<Mutex<HashMap>>
            }
            node.is_failed = true;
            println!("Host {} is marked as failed", max_id);
        }
    }
}




pub async fn bully_algorithm_init(servers: Arc<Mutex<HashMap<u32, Node>>>, my_id: u32, socket_a: &Socket, config: &Config) {

    let socket = socket_a.clone();
    let start_time = time::Instant::now();
    let timeout_duration = Duration::from_secs(5);

    println!("Host {} initializing bully algorithm.", my_id);

    let (tx, mut rx): (broadcast::Sender<(u32, u32)>, broadcast::Receiver<(u32, u32)>) = 
        broadcast::channel(10); // Local in-memory message passing
    let listening_tx = tx.clone();
    let socket_listener = Arc::clone(&socket.socket_failover_tx);

    // Spawn listener task
    tokio::task::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            let result = {
                let listener = socket_listener.lock().await; // Lock the UdpSocket
                timeout(timeout_duration, listener.recv_from(&mut buf)).await
            };

            match result {
                Ok(Ok((size, addr))) => {
                    if let Ok((id, random_number)) = bincode::deserialize::<(u32, u32)>(&buf[..size]) {
                        println!("Received message from {}: (id: {}, random_number: {})", addr, id, random_number);
                        if id != my_id{
                            let _ = listening_tx.send((id, random_number)); // Broadcast message

                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Error receiving message: {:?}", e);
                }
                Err(_) => {
                    println!("Timeout reached with no messages received within {:?}", timeout_duration);
                    break; 
                }
            }
        }
    });

    let random_number = rand::thread_rng().gen_range(0..100);
    println!("Host {} generated random identifier: {}", my_id, random_number);
    send_random(&socket, my_id, &config, random_number).await;

    let mut numbers = vec![(my_id, random_number)];
    let mut received_hosts = HashSet::new();
    received_hosts.insert(my_id);


    while start_time.elapsed() < timeout_duration {
        match time::timeout(timeout_duration - start_time.elapsed(), rx.recv()).await {
            Ok(Ok((id, random_number))) => {
                if !received_hosts.contains(&id) {
                    numbers.push((id, random_number));
                    received_hosts.insert(id);
                }
            }
            _ => break, // Break on timeout or error
        }
    }
    if numbers.len() == 1 {        
        let mut servers_clone = servers.lock().await;
        for node in servers_clone.values_mut() {
            node.is_failed = false;
        }
        return;
    }

    let max_host = numbers.into_iter().max_by_key(|(_, number)| *number);

    
    if let Some((max_id, _)) = max_host {
        if max_id == my_id {
            println!("Host {}: failed", my_id);
        } else {
            println!("Host {}: active", my_id);
        }

        let mut servers_clone = servers.lock().await;

        for node in servers_clone.values_mut() {
            node.is_failed = false;
        }

        if let Some(node) = servers_clone.get_mut(&max_id) {
            if max_id == my_id && node.is_dos_leader {
                let _ = elect_leader_dos(servers.clone(), my_id, &socket, config).await; // Pass the Arc<Mutex<HashMap>>
            }
            node.is_failed = true;
            println!("Host {} is marked as failed", max_id);
        }
    }
}


pub async fn bully_listener( servers: Arc<Mutex<HashMap<u32, Node>>>, my_id: u32, socket_a: &Arc<Socket>, config: &Arc<Config>, is_running: Arc<Mutex<bool>>) {
    
    let failover_socket = Arc::clone(&socket_a.socket_failover_tx);

    let mut buf = [0u8; 1024];
    let mut interval = tokio::time::interval(Duration::from_secs(30)); // Interval for periodic tasks
    println!("Waiting for sim fail message");
    loop {
        tokio::select! {
            _ = interval.tick() => {
                        let servers_clone = Arc::clone(&servers);
                        let socket_clone = Arc::clone(&socket_a);  // Clone Arc here
                        let config_clone = Arc::clone(&config);    // Clone Arc here
                        bully_algorithm_init(servers_clone, my_id, &socket_clone, &config_clone).await;
                    }
                }
            // result = async {
            //     let binding = failover_socket.clone();
            //     let socket = binding.lock().await;
            //     socket.recv_from(&mut buf).await
            // } => {
            //     match result {
            //         Ok((size, _)) => {
            //             if let Ok((id, random_number)) = bincode::deserialize::<(i32, u32)>(&buf[..size]) {
            //                 if id as u32 != my_id {
            //                     println!("Starting bully algorithm due to received message: {:?}", (id, random_number));
            //                     let servers_clone = Arc::clone(&servers);
            //                     let socket_clone = Arc::clone(&socket_a);  // Clone Arc here
            //                     let config_clone = Arc::clone(&config);    // Clone Arc here
            //                     bully_algorithm(servers_clone, my_id, &socket_clone, &config_clone,(id,random_number)).await;  
                                
            //                 } else {
            //                     println!("Received own message, skipping.");
            //                 }
            //             }
            //         },
            //         Err(e) => {
            //             eprintln!("Error receiving message: {:?}", e);
            //         }
            //     }
            // }
        //}
    }
}

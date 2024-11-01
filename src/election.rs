use std::sync::Arc;
use tokio::task;
use tokio::{net::UdpSocket as TokioUdpSocket, sync::broadcast, time};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HostMessage {
    id: u32,
    random_number: u32,
    failed: bool,
}


//algorithm takes in a port number, an id you specify (mostly for debugging), and an array of strings which is the other hosts' ips
pub async fn bully_algorithm(port: u16, curr_id: u32, other_hosts: Vec<String>,socket_l:Arc<TokioUdpSocket>) {

    let curr_addr = format!("0.0.0.0:{}", port);

    let socket = socket_l;

    let num_hosts = other_hosts.len();

    println!("Host {} listening on {}", curr_id, curr_addr);

    let (tx, mut rx) = broadcast::channel(10); //channel can hold 10 messages, should be enough since its fast
    let listening_tx = tx.clone();

    let socket_listener = socket.clone();

    //this is in charge of receiving and broadcasting the received messages
    task::spawn(async move {
        //time::sleep(Duration::from_secs(1)).await;
        let mut buf = [0u8; 1024]; //1024 length array of u8 initilized to 0's
        //if we receive correctly from the socket_listener we get a tuple of (size, address)
        //in the case of success go to the next step
        while let Ok((size, addr)) = socket_listener.recv_from(&mut buf).await {
            //the receivied item should be deserialziable into a "hostmessage" struct
            //in that case we add it to the buffer print it for debugging and finally broadcast it 
            if let Ok(msg) = bincode::deserialize::<HostMessage>(&buf[..size]) {
                println!("received message from {}: {:?}", addr, msg);
                let _ = listening_tx.send(msg);
            }
        }
    });

    let random_number = rand::thread_rng().gen_range(0,100); //identifier for the failing
    println!("Host {} generated identifier: {}", curr_id, random_number);

    let election_socket = socket.clone(); //this is to send our own info

    task::spawn(async move {
        let msg = HostMessage { id: curr_id, random_number, failed: false }; //send the struct that defines the current host
        let msg_bytes = bincode::serialize(&msg).unwrap(); //serialize before sending

        for host_addr in &other_hosts {
            let _ = election_socket.send_to(&msg_bytes, host_addr).await;
        }
    });


    let mut numbers = vec![(curr_id, random_number)]; //array of tuples
    let mut received_hosts = HashSet::new(); //using this to check if i already have the host, because it might be broadcasted more than once
    received_hosts.insert(curr_id);
    let start_time = time::Instant::now();
    
    let timeout_duration =  Duration::from_secs(5);
    let remaining_time = timeout_duration -  start_time.elapsed(); //using a timeout of three seconds, idk if thats too little

   
    while start_time.elapsed() < timeout_duration { 
        match time::timeout(remaining_time, rx.recv()).await {
            Ok(Ok(msg)) => {
                if !received_hosts.contains(&msg.id) {// Check if received before
                    numbers.push((msg.id, msg.random_number)); //push to the vec 
                    received_hosts.insert(msg.id); //add to the checking hashset
                }
            }
            _ => break, //if timed out (or some error) leave the loop
        }
        
        if received_hosts.len() == num_hosts + 1 { //if were done just leave to save time
            break;
        }
    }

    let max_host = numbers.iter().max_by_key(|x| x.1).unwrap();
    let failed = max_host.0 == curr_id;
    println!("Host {}: {}", curr_id, if failed { "failed" } else { "active" });

}


pub async fn bully_listener(port: u16, curr_id: u32, other_hosts: Vec<String>) {
    let listener_addr = format!("0.0.0.0:{}", port);
    let socket = Arc::new(TokioUdpSocket::bind(&listener_addr).await.unwrap());

    let mut buf = [0u8; 1024];

    loop {
        let (size, _) = socket.recv_from(&mut buf).await.unwrap();
        if let Ok(msg) = bincode::deserialize::<HostMessage>(&buf[..size]) {
            println!("Starting sim fail due to received message: {:?}", msg);
            let _ = bully_algorithm(port, curr_id, other_hosts.clone(),socket.clone()).await; 
        }
    }
}
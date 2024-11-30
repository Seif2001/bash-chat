
mod leader;
pub mod config;
pub mod socket;
pub mod com;
pub mod image_com;
pub mod failure;
pub mod dos;
use crate::leader::Node;
use crate::config::Config;
use crate::socket::Socket;
use std::net::Ipv4Addr;

use std::sync::{Arc};
use tokio::sync::Mutex;
use std::str::FromStr;

mod image_processor;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    
    let servers: Arc<Mutex<std::collections::HashMap<u32, Node>>> = Arc::new(Mutex::new(
        vec![
            (0, Node{is_leader: true, is_dos_leader:true, is_failed: false, current_leader: 0, current_dos_leader:0, term: 0}),
            // (1, Node{is_leader: false, is_dos_leader:false, is_failed: false, current_leader: 0, current_dos_leader:0, term: 0}),
            // (2,Node{is_leader: false, is_dos_leader:false, is_failed: false, current_leader: 0, current_dos_leader:0, term: 0}),
        ].into_iter().collect()
    ));

    let my_id = 0;
    let config = Config::new();

    let socket = Socket::new(config).await;
    let socket_arc = Arc::new(socket);
    com::join(&socket_arc.socket_election_tx, &socket_arc.socket_failover_tx).await.expect("Failed to join multicast group");
    let config = Config::new();
    let config_arc = Arc::new(config); 

    let server_clone = servers.clone();


    leader::elections(server_clone.clone(), my_id, &socket_arc, &config_arc).await;
    leader::elections_dos(servers, &socket_arc).await;
    dos::dos_registrar(server_clone.clone(),my_id, &socket_arc, &config_arc).await;
    dos::recv_dos(&socket_arc, &config_arc).await;
   // Spawn the image receiving task
   tokio::spawn({
    let socket_arc = Arc::clone(&socket_arc);  // Clone the Arc to move into the task
    let config_arc = Arc::clone(&config_arc);  // Clone the Arc to move into the task

    async move {
        // This runs in the spawned task
        match image_com::receive_image(&socket_arc, &config_arc).await {
            Ok(_) => {
                println!("Image received successfully.");
            }
            Err(e) => {
                eprintln!("Error receiving image: {}", e);
            }
        }
    }
});


    //let add: Ipv4Addr = Ipv4Addr::from_str("192.168.1.2").expect("Invalid IP address");
    //dos::update_dos(add,"test123".to_string()).await;


    // Spawn the bully listener task in a separate thread
    let listener_task = tokio::spawn({
        let server_clone = server_clone.clone();
        let socket_clone = socket_arc.clone(); // Clone the Arc
        let config_clone = config_arc.clone(); // Clone the Arc
        async move {
            failure::bully_listener(server_clone, my_id, &socket_clone, &config_clone, Arc::new(Mutex::new(false))).await;
        }
    });
    

    loop{
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

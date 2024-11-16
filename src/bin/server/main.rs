
mod leader;
pub mod config;
pub mod socket;
pub mod com;
pub mod failure;
use crate::leader::Node;
use crate::config::Config;
use crate::socket::Socket;

use std::sync::{Arc};
use tokio::sync::Mutex;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    
    let servers: Arc<Mutex<std::collections::HashMap<u32, Node>>> = Arc::new(Mutex::new(
        vec![
            (0, Node{is_leader: true, is_failed: false, current_leader: 0, term: 0}),
            (1, Node{is_leader: false, is_failed: false, current_leader: 0, term: 0}),
            (2,Node{is_leader: false, is_failed: false, current_leader: 0,  term: 0}),
        ].into_iter().collect()
    ));

    let my_id = 0;
    let config = Config::new();

    let socket = Socket::new(config.address_election_tx, config.address_election_rx, config.address_failover_tx, config.address_failover_rx, config.address_client).await;
    let socket_arc = Arc::new(socket);
    com::join(&socket_arc.socket_election_tx, &socket_arc.socket_failover_tx).await.expect("Failed to join multicast group");
    let config = Config::new();
    let config_arc = Arc::new(config); 

    let _ = leader::recv_leader(&socket_arc, servers.clone()).await;
    let server_clone = servers.clone();


    leader::elections(server_clone.clone(), my_id, &socket_arc, &config_arc).await;

// Spawn the bully listener task in a separate thread
    // let listener_task = tokio::spawn({
    //     let server_clone = server_clone.clone();
    //     let socket_clone = socket_arc.clone(); // Clone the Arc
    //     let config_clone = config_arc.clone(); // Clone the Arc
    //     async move {
    //         failure::bully_listener(server_clone, my_id, &socket_clone, &config_clone, Arc::new(Mutex::new(false))).await;
    //     }
    // });
    

    loop{
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

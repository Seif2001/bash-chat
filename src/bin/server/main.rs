
mod leader;
pub mod config;
pub mod socket;
pub mod com;
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
            (0, Node{is_leader: true, is_failed: false}),
            (1, Node{is_leader: false, is_failed: false}),
            (2, Node{is_leader: false, is_failed: false}),
        ].into_iter().collect()
    ));

    let my_id = 0;
    let config = Config::new();

    let socket = Socket::new(config.address_election_tx, config.address_election_rx, config.address_failover_tx, config.address_failover_rx, config.address_client).await;
    com::join(&socket.socket_election_tx, &socket.socket_failover_tx).await.expect("Failed to join multicast group");
    let config = Config::new();

    let _ = leader::recv_leader(&socket).await;
    leader::elections(servers, my_id, &Arc::new(socket), &Arc::new(config)).await;
    // run tokio join for recieving leader messages
    loop{
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

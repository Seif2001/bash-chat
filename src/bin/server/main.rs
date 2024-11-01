mod server; // Include the server module
mod middleware;
use std::sync::{Arc, Mutex, RwLock};
use tokio::time::{Duration, sleep};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the Server
    let server = server::Server::new(2, false, Arc::new(RwLock::new(2))).await;

    // Join multicast group
    let server = Arc::new(server);
    server.clone().join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    let server = server.clone();
    server.election().await.expect("Failed to elect leader"); 
      


    Ok(())
}

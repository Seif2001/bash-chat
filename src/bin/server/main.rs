mod server; // Include the server module
mod middleware;
use std::sync::{Arc, Mutex, RwLock};
use tokio::time::{Duration, sleep};
use std::sync::atomic::{AtomicU32, Ordering};



#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the Server
    let server = server::Server::new(3, false, Arc::new(RwLock::new(3)) ).await;

    // Join multicast group
    let server = Arc::new(server);
    server.clone().join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    let server_clone = server.clone();

    server_clone.election().await.expect("Failed to elect leader");
    //let server_clone = server.clone();
    //server_clone.bully_listener().await;

    //add loop to keep the server running
    // loop {
    //     sleep(Duration::from_secs(1)).await;
    // }
       


    Ok(())
}




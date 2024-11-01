mod server; // Include the server module
mod middleware;
use std::sync::{Arc, Mutex, RwLock};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the Server
    let server = server::Server::new(1, false, Arc::new(RwLock::new(1))).await;

    // Join multicast group
    let server = Arc::new(server);
    server.clone().join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    loop{
        let server = server.clone();
        tokio::spawn(async move{
            server.election().await.expect("Failed to elect leader"); 
        });
    }


    Ok(())
}

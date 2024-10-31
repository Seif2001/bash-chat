mod server; // Include the server module
mod middleware;
use std::sync::{Arc, Mutex};


#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the Server
    let server = server::Server::new(1, false, 1).await;

    // Join multicast group
    let server = Arc::new(server);
    server.clone().join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    server.clone().recv_rpc().await.expect("Failed to setup RPC receiving");
    //server.process_client().await.expect("Failed to setup client processing");

    Ok(())
}

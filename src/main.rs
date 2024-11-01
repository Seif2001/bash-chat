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
    let recv_task = {
        let server = server.clone();
        tokio::spawn(async move {
            server.recv_rpc().await;
        })
    };    //server.process_client().await.expect("Failed to setup client processing");
    let _ = tokio::join!(recv_task);
    server.clone().send_info().await.expect("Failed to setup RPC receiving");

    Ok(())
}
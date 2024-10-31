mod server; // Include the server module
mod middleware;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the Server
    let server = server::Server::new(1, false, 1).await;

    // Join multicast group
    server.join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    //server.send_rpc().await.expect("Failed to setup RPC sending");
    server.recv_rpc().await.expect("Failed to setup RPC receiving");
    //server.process_client().await.expect("Failed to setup client processing");

    Ok(())
}

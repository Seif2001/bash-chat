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
    //let server_clone = server.clone();

    //server_clone.election().await.expect("Failed to elect leader");


    //dont wait for bullt listener star a new task

    let server_clone = server.clone();
    tokio::task::spawn(async move {
        server_clone.bully_listener().await;
    });


    //add loop to keep the server running
    loop {
        sleep(Duration::from_secs(5)).await;
    
    }
       


    Ok(())
}




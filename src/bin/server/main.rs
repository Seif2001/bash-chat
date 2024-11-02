mod server; // Include the server module
mod middleware;
use std::sync::{Arc, Mutex, RwLock};
use tokio::time::{Duration, self};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::atomic::AtomicBool;




#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the Server
    let server = server::Server::new(3, false, Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)) ).await;

    // Join multicast group
    let server = Arc::new(server);
    server.clone().join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    let server_clone = server.clone();

    tokio::spawn(async move{
        loop{
            server_clone.clone().send_leader().await.expect("Failed");
            time::sleep(Duration::from_secs(1)).await;
        }
    });

    let server_clone = server.clone();

    tokio::spawn(async move{

        server_clone.clone().recv_leader().await.expect("Failed to recieve");
    });

    let server_clone = server.clone();

    tokio::spawn(async move{
        loop{
            server_clone.clone().send_info().await.expect("Failed to recieve");
        }
    });
    let server_clone = server.clone();

    tokio::spawn(async move{

        server_clone.clone().recv_election().await.expect("Failed to recieve");
    });
    let server_clone = server.clone();

    tokio::spawn(async move{
        time::sleep(Duration::from_secs(5)).await;
        server_clone.clone().start_election().await.expect("Failed to recieve");
    }); 
    loop{
        time::sleep(Duration::from_secs(1)).await;
    }
    
    //let server_clone = server.clone();
    //server_clone.bully_listener().await;
    
    


    Ok(())
}




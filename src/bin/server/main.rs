use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::time::{Duration, self};

mod server;
mod image_processor;
mod middleware;


use std::sync::atomic::AtomicBool;

#[tokio::main]
async fn main() -> io::Result<()> {

     // Initialize the Server
     let server = server::Server::new(3, false, Arc::new(AtomicBool::new(true)), Arc::new(AtomicBool::new(false)) ).await;

     // Join multicast group
    let server = Arc::new(server);
    server.clone().join().await.expect("Failed to join multicast group");

    // Set up tasks for sending, receiving, and processing client messages
    let server_clone = server.clone();

   let task_send_leader = tokio::spawn(async move{
        loop{
            server_clone.clone().send_leader().await.expect("Failed");
            time::sleep(Duration::from_secs(1)).await;
        }
    });

    let server_clone = server.clone();

    let task_recv_leader =tokio::spawn(async move{

        server_clone.clone().recv_leader().await.expect("Failed to recieve");
    });
    let server_clone = server.clone();
    let task_recv_start = tokio::spawn(async move{
        loop{
            server_clone.clone().recieve_start_elections().await.expect("Failed to send message");
        }
    });

    let server_clone = server.clone();
    let task_send_info =tokio::spawn(async move{
        loop{
            server_clone.clone().send_info().await.expect("Failed to send message");
        }
    });

    let server_clone = server.clone();
    let task_recv_elections = tokio::spawn(async move{
            server_clone.clone().recv_election().await.expect("Failed to send message");
    });

    let server_clone = server.clone();

    
    let server_a = server.clone();
    let image_data = server_a.initialize_image_buffer();
    let client_addr = server_a.initialize_client_addr();
    
    // middle ware function
    server_a.start_receive_task(image_data.clone(), client_addr.clone()).await;
    tokio::try_join!(task_send_leader, task_recv_leader, task_recv_start, task_send_info, task_recv_elections).expect("Failed to join tasks");
    server_a.wait_for_shutdown().await;
    Ok(())
}

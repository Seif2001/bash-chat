use std::sync::{Arc, Mutex};
use tokio::io;
mod server;
mod image_processor;
mod middleware;

//seif main



// #[tokio::main]
// async fn main() -> std::io::Result<()> {
//     dotenv::dotenv().ok();

//     // Initialize the Server
//     let server = server::Server::new(1, false, 1).await;

//     // Join multicast group
//     let server = Arc::new(server);
//     server.clone().join().await.expect("Failed to join multicast group");

//     // Set up tasks for sending, receiving, and processing client messages
//     let recv_task = {
//         let server = server.clone();
//         tokio::spawn(async move {
//             server.recv_rpc().await;
//         })
//     };    //server.process_client().await.expect("Failed to setup client processing");
//     let _ = tokio::join!(recv_task);
//     server.clone().send_info().await.expect("Failed to setup RPC receiving");

//     Ok(())
// }


//moneer main

#[tokio::main]
async fn main() -> io::Result<()> {

    let server_a = server::Server::new(1, false, 1).await; //1 notfailed leader

    let server_ip = "127.0.0.1:".to_string();

    let image_data = server_a.initialize_image_buffer();
    let client_addr = server_a.initialize_client_addr();
    
    // middle ware function
    server_a.start_receive_task(image_data.clone(), client_addr.clone()).await;
    server_a.wait_for_shutdown().await
}

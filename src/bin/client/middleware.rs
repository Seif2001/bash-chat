use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;

// send to 3 servers

pub async fn send_cloud(socket: &Socket, config: &Config)  -> std::io::Result<()>  {
    let socket_server_1 = &socket.socket_server_1;
    let socket_server_2 = &socket.socket_server_2;
    let socket_server_3 = &socket.socket_server_3;

    let message = "START";

    // Verify destination addresses before sending
    println!("Sending message to server 1: {}:{}", config.server_ip_1, config.port_server_rx);
    println!("Sending message to server 2: {}:{}", config.server_ip_2, config.port_server_rx);
    println!("Sending message to server 3: {}:{}", config.server_ip_3, config.port_server_rx);

    // Task to send message to server 1
    let send_to_server_1 = {
        let socket_server_1_clone = socket_server_1.clone();
        let server_ip_1 = config.server_ip_1.clone();
        let port_send_1 = config.port_server_rx;
        tokio::spawn(async move {
            let dest = (server_ip_1, port_send_1);
            if let Err(e) = com::send(&socket_server_1_clone, message.to_string(), dest).await {
                eprintln!("Failed to send message to server 1: {}", e);
            }
        })
    };

    // Task to send message to server 2
    let send_to_server_2 = {
        let socket_server_2_clone = socket_server_2.clone();
        let server_ip_2 = config.server_ip_2.clone();
        let port_send_2 = config.port_server_rx;
        tokio::spawn(async move {
            let dest = (server_ip_2, port_send_2);
            if let Err(e) = com::send(&socket_server_2_clone, message.to_string(), dest).await {
                eprintln!("Failed to send message to server 2: {}", e);
            }
        })
    };

   
    let send_to_server_3 = {
        let socket_server_3_clone = socket_server_3.clone();
        let server_ip_3 = config.server_ip_3.clone();
        let port_send_3 = config.port_server_rx;
        tokio::spawn(async move {
            let dest = (server_ip_3, port_send_3);
            if let Err(e) = com::send(&socket_server_3_clone, message.to_string(), dest).await {
                eprintln!("Failed to send message to server 3: {}", e);
            }
        })
    };

    // Wait for all tasks to complete
    let _ = tokio::try_join!(send_to_server_3, send_to_server_2, send_to_server_1)?;
    Ok(())
}

// receive leader message

pub async fn recv_leader(socket: &Socket) -> std::io::Result<()> {
    let socket = socket.socket_client_leader_rx.clone();

    tokio::spawn(async move {
        loop {
            let (message, src) = com::recv(&socket).await.expect("Failed to receive message");
            let message = message.trim();
            let src = src.ip();
            
            if message.starts_with("LEADER") {
               println!("Received leader message from {}: {}", src, message);
                
            }
        }
    });
    Ok(())
}
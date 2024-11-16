use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;
use std::net::Ipv4Addr;

// send to leader image

pub async fn send_image(server_address: Ipv4Addr, socket: &Socket, config: &Config)  -> std::io::Result<()>  {
    let socket_client_server_tx = &socket.socket_client_server_tx;

    let message = "hello from client";

    // Verify destination addresses before sending
    println!("Sending message to leader: {}:{}", server_address, config.port_server_rx);

    com::send(socket_client_server_tx, message.to_string(), (server_address, config.port_server_rx)).await?;
    Ok(())
}
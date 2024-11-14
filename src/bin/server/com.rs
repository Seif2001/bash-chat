use std::io;
use std::net::{SocketAddr, Ipv4Addr};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex};
use std::sync::Arc;

use crate::config::Config;

pub async fn join(socket_election_tx: &Arc<Mutex<UdpSocket>>, socket_failover_tx: &Arc<Mutex<UdpSocket>>) -> std::io::Result<()> {
    let socket_election_tx = socket_election_tx.lock().await;
    let socket_failover_tx = socket_failover_tx.lock().await;
    let config = Config::new();
    let multicast_addr = config.multicast_addr;
    let interface_addr = config.interface_addr;
    socket_election_tx.join_multicast_v4(multicast_addr,interface_addr)?;
    socket_failover_tx.join_multicast_v4(multicast_addr,interface_addr)?;       
    Ok(())
}

//send(socket, message, dest)


pub async fn send(socket: &Arc<Mutex<UdpSocket>>, message: String, dest: (std::net::Ipv4Addr, u16)) -> std::io::Result<()> {
    let socket = socket.lock().await;
    socket.send_to(message.as_bytes(), dest).await?;


    Ok(())

}
//recv(socket)

pub async fn recv(socket: &Arc<Mutex<UdpSocket>>) -> io::Result<(String, SocketAddr)> {
    let socket = socket.lock().await;
    let mut buf = [0; 1024];

    // Receive data from the UDP socket
    let (amt, src) = socket.recv_from(&mut buf).await?;

    // Convert the received data into a String and return the message and source address
    let message = String::from_utf8_lossy(&buf[..amt]).to_string();
    Ok((message, src))
}





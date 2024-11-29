use std::io;
use std::net::{SocketAddr, Ipv4Addr};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex};
use std::sync::Arc;

use crate::config::Config;

pub async fn send(socket: &Arc<Mutex<UdpSocket>>, message: String, dest: (std::net::Ipv4Addr, u16)) -> std::io::Result<()> {
    let socket = socket.lock().await;
    println!("Sending message: {} to {}:{}", message, dest.0, dest.1);
    socket.send_to(message.as_bytes(), dest).await?;

    Ok(())

}


pub async fn recv(socket: &Arc<Mutex<UdpSocket>>) -> io::Result<(String, SocketAddr)> {
    let socket = socket.lock().await;
    let mut buf = [0; 1024];

    // Receive data from the UDP socket
    let (amt, src) = socket.recv_from(&mut buf).await?;

    // Convert the received data into a String and return the message and source address
    let message = String::from_utf8_lossy(&buf[..amt]).to_string();
    Ok((message, src))
}

pub async fn recv_raw(socket: &Arc<Mutex<UdpSocket>>) -> io::Result<([u8; 1024], SocketAddr)> {
    let socket = socket.lock().await;
    let mut buf = [0; 1024];

    // Receive data from the UDP socket
    let (amt, src) = socket.recv_from(&mut buf).await?;

    Ok((buf, src))
}

pub async fn recv_raw_size(socket: &Arc<Mutex<UdpSocket>>) -> io::Result<([u8; 1024], SocketAddr, u32)> {
    let socket = socket.lock().await;
    let mut buf = [0; 1024];

    // Receive data from the UDP socket
    let (amt, src) = socket.recv_from(&mut buf).await?;

    Ok((buf, src, amt as u32))
}



pub async fn send_vec(socket: &Arc<Mutex<UdpSocket>>, message: Vec<u8>, dest: (std::net::Ipv4Addr, u16)) -> std::io::Result<()> {
    let socket = socket.lock().await;
    socket.send_to(&message, dest).await?;
    Ok(())
}


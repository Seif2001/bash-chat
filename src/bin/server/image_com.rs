use async_std::path::{Path, PathBuf};

use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::com;
use std::net::Ipv4Addr;
use async_std::sync::{Arc};
use std::convert::TryInto;
use tokio::sync::Mutex;


pub async fn recv_image_name(socket: &Socket, config: &Config) -> Result<(PathBuf, Ipv4Addr), std::io::Error> {
    let socket_server_client_rx = &socket.socket_server_client_rx;
    let socket_server_client_tx = &socket.socket_server_client_tx;
    let (image_name, src) = com::recv(&socket_server_client_rx).await?;
    let image_name = image_name.trim();
    println!("Received message: {}", image_name);
    let image_path = Path::new(&config.raw_images_dir).join(&image_name);
    
    let client_ip = match src {
        std::net::SocketAddr::V4(addr) => *addr.ip(),
        std::net::SocketAddr::V6(addr) => {
            panic!("Expected Ipv4Addr but got Ipv6Addr: {}", addr)
        }
    };
    let image_path = image_path.clone();   
    let dest = (
        src.ip()
            .to_string()
            .parse::<Ipv4Addr>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid IP address"))?,
        src.port(),
    ); 
    com::send(&socket_server_client_tx, "NAME_ACK".to_string(), (dest)).await?;
    Ok((image_path.to_path_buf(), client_ip))
}

pub async fn recv_image_chunk(
    socket: &Socket,
    config: &Config,
    expected_chunk_index: u32,
    image_data: Arc<Mutex<Vec<u8>>>,
) -> Result<u32, std::io::Error> {
    let socket_server_rx = socket.socket_server_client_rx.clone();
    let socket_server_tx = socket.socket_server_client_tx.clone();
    
    let (buf, src) = com::recv(&socket_server_rx).await?;
    let len = buf.len();

    let chunk_index = u32::from_be_bytes(
        buf[..4]
            .as_bytes()
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk index format"))?,
    );

    if chunk_index == expected_chunk_index {
        image_data
            .lock()
            .await
            .extend_from_slice(&buf.as_bytes()[4..len]);

        let dest = (
            src.ip()
                .to_string()
                .parse::<Ipv4Addr>()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid IP address"))?,
            src.port(),
        );
        com::send(&socket_server_tx, chunk_index.to_string(), dest)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send ACK: {}", e)))?;

        return Ok(expected_chunk_index + 1);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("Expected chunk index {} but got {}", expected_chunk_index, chunk_index),
    ))
}


// pub async fn recv_name_chunk(socket: &Socket, config: &Config) -> (String, u32) {
    
//     loop{
//         let (image_path, client_ip) = recv_image_name(socket, config).await;
        
//     }

//     return 
// }

// send image

// recv ack

// send ack

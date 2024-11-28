use async_std::path::{Path, PathBuf};
use crate::{com, image_processor};
use crate::config::{self, Config};
use crate::socket::{self, Socket};
use std::net::Ipv4Addr;
use async_std::sync::Arc;
use async_std::fs::File;
use async_std::io::prelude::*;
use std::convert::TryInto;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

use image_processor::encode_image;

//Receiving
pub async fn recv_image_name(socket: &Socket, config: &Config) -> Result<(PathBuf, Ipv4Addr), std::io::Error> {
    let socket_server_client_rx = &socket.socket_client_rx;
    let socket_server_client_tx = &socket.socket_client_tx;
    let (image_name, src) = com::recv(&socket_server_client_rx).await?;
    let image_name = image_name.trim();
    println!("Received message: {}", image_name);
    let image_path = Path::new(&config.server_raw_images_dir).join(&image_name);
    
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
    println!("dest: {:?}", dest);
    com::send(&socket_server_client_rx, "NAME_ACK".to_string(), (dest)).await?;
    Ok((image_path.to_path_buf(), client_ip))
}

pub async fn recv_image_chunk(
    socket: &Socket,
    config: &Config,
    expected_chunk_index: u32,
    image_data: Arc<Mutex<Vec<u8>>>,
) -> Result<Option<u32>, std::io::Error> {
    let socket_server_rx = socket.socket_server_client_rx.clone();
    let socket_server_tx = socket.socket_server_client_tx.clone();
    
    let (buf, src) = com::recv_raw(&socket_server_rx).await?;
    let len = buf.len();
    println!("Received chunk: {:?}", len);

    if &buf[..3] == b"END" {
        println!("Received 'END' marker. Transmission completed.");
        return Ok(None); // Signal the end of transmission
    }
    let chunk_index = u32::from_be_bytes(
        buf[..4]
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid chunk index format"))?,
    );

    if chunk_index == expected_chunk_index {
        image_data
            .lock()
            .await
            .extend_from_slice(&buf[4..len]);

        let dest = (
            src.ip()
                .to_string()
                .parse::<Ipv4Addr>()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid IP address"))?,
            src.port(),
        );
        com::send(&socket_server_tx, chunk_index.to_string(), (dest))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send ACK: {}", e)))?;

        return Ok(Some(expected_chunk_index + 1));
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("Expected chunk index {} but got {}", expected_chunk_index, chunk_index),
    ))
}

pub async fn receive_image(
    socket: &Socket,
    config: &Config,
) -> Result<(), std::io::Error> {
    // Step 1: Receive the image name
    let (image_path, client_ip) = recv_image_name(socket, config).await?;
    println!("Image name received: {:?}", image_path);

    // Prepare for receiving image chunks
    let image_data = Arc::new(Mutex::new(Vec::new()));
    let mut expected_chunk_index = 0;

    // Step 2: Receive image chunks
    loop {
        match recv_image_chunk(socket, config, expected_chunk_index, image_data.clone()).await? {
            Some(next_chunk_index) => {
                expected_chunk_index = next_chunk_index; // Continue to the next chunk
            }
            None => {
                // "END" marker detected; save the image and exit
                println!("Saving received image...");
                
                // Lock the image data to get all received chunks
                let image_data_locked = image_data.lock().await;
                
                // Define the path to save the image
                // let image_path = format!("/home/g7/Desktop/yehia/Distributed/bash-chat/src/bin/server/raw_images/{}", image_path.to_string_lossy());
    
                // Create and write the file
                let mut file = File::create(image_path.clone()).await?;
                file.write_all(&image_data_locked).await?;
                file.flush().await?;

                println!("Image saved at: {:?}", image_path);

                println!("Encoding the image...");
                let encoded_dir = Path::new(&config.server_encoded_images_dir);
                let mask_image_path = "./src/bin/server/masks/mask2.jpg";
                // Define the output image path in the decoded images directory
                let output_image_path = format!("{}/{}", encoded_dir.display(), Path::new(&image_path).file_name().unwrap().to_string_lossy());
                println!("output_image_path: {}", output_image_path);
                let _ = encode_image(image_path.to_string_lossy().to_string(), output_image_path.to_string(), mask_image_path.to_string());

                //send the encoded image to the client
                println!("Sending the encoded image to the client...");
                send_image(socket, Path::new(&output_image_path), client_ip, config.port_client_rx, 1020).await?;
                println!("Encoded image sent to the client.");
                break;
            }
        }
    }

    Ok(())
}

//Sending
pub async fn send_image_name(
    socket: &Socket,
    image_name: &str,
    client_ip: Ipv4Addr,
    client_port: u16,
) -> Result<(), std::io::Error> {
    let dest = (client_ip, client_port);
    println!("Sending image name: {}", image_name);
    com::send(&socket.socket_server_client_tx, image_name.to_string(), dest).await?;
    println!("Sending image name: {}", image_name);
    // Wait for "NAME_ACK"
    let (ack, _) = com::recv(&socket.socket_server_client_tx).await?;
    if ack.trim() != "NAME_ACK" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to receive NAME_ACK",
        ));
    }

    println!("Image name '{}' sent and acknowledged.", image_name);
    Ok(())
}
// ub async fn send_image_chunk(
//     socket: &Socket,
//     chunk_data: &[u8],
//     chunk_index: u32,
//     server_ip: Ipv4Addr,
//     server_port: u16,
// ) -> Result<(), std::io::Error> {
//      // Prepare the chunk index as a byte array
//      let mut data_with_index = chunk_index.to_be_bytes().to_vec();
//      // Append the chunk data
//      data_with_index.extend_from_slice(chunk_data);
     
//     let dest = (server_ip, 6376);
    
//     println!("size of chunk: {}", data_with_index.len());
//     com::send_vec(&socket.socket_client_server_tx, data_with_index, dest).await?;
//     println!("Chunk {} sent.", chunk_index);
//     let (ack, _) = com::recv(&socket.socket_client_server_tx).await?;
//     let ack_index: u32 = ack
//         .trim()
//         .parse()
//         .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ACK received"))?;

//     if ack_index != chunk_index {
//         return Err(std::io::Error::new(
//             std::io::ErrorKind::InvalidData,
//             format!(
//                 "Chunk index mismatch: expected {}, got {}",
//                 chunk_index, ack_index
//             ),
//         ));
//     }

//     println!("Chunk {} sent and acknowledged.", chunk_index);
//     Ok(())
// }

pub async fn send_image_chunk(
    socket: &Socket,
    chunk_data: &[u8],
    chunk_index: u32,
    client_ip: Ipv4Addr,
    client_port: u16,
) -> Result<(), std::io::Error> {
    let timeout_duration: Duration = Duration::from_secs(1);
    let max_retries: u32 = 3;
    let mut retries = 0;
    loop {
        println!("Sending chunk {} (attempt {})...", chunk_index, retries + 1);
        let mut data_with_index = chunk_index.to_be_bytes().to_vec();
        data_with_index.extend_from_slice(chunk_data);

        let dest = (client_ip, client_port);
        com::send_vec(&socket.socket_server_client_tx, data_with_index.clone(), dest).await?;

        // Wait for ACK with a timeout
        match time::timeout(timeout_duration, com::recv(&socket.socket_server_client_tx)).await {
            Ok(Ok((ack, _))) => {
                // Parse the ACK index
                let ack_index: u32 = ack
                    .trim()
                    .parse()
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ACK received")
                    })?;

                if ack_index == chunk_index {
                    println!("Chunk {} sent and acknowledged.", chunk_index);
                    return Ok(());
                } else {
                    println!(
                        "ACK index mismatch: expected {}, got {}. Retrying...",
                        chunk_index, ack_index
                    );
                }
            }
            Ok(Err(err)) => {
                println!("Error receiving ACK for chunk {}: {}. Retrying...", chunk_index, err);
            }
            Err(_) => {
                println!(
                    "Timeout waiting for ACK for chunk {}. Retrying...",
                    chunk_index
                );
            }
        }

        retries += 1;

        if retries >= max_retries {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("Failed to receive ACK for chunk {} after {} retries", chunk_index, retries),
            ));
        }
    }
}

pub async fn send_image(
    socket: &Socket,
    image_path: &Path,
    client_ip: Ipv4Addr,
    client_port: u16,
    chunk_size: usize,
) -> Result<(), std::io::Error> {
    // Extract the image name
    let image_name = image_path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid image path")
    })?;
    let image_name = image_name.to_string_lossy();

    // Step 1: Send the image name
    println!("Sending image name: {}", image_name);
    send_image_name(socket, &image_name, client_ip, client_port).await?;

    // Step 2: Open the image file and send chunks
    let mut file = File::open(image_path).await?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).await?;
    println!("File read into memory, size: {} bytes", file_contents.len());

    let mut chunk_index: u32 = 0;

    for chunk in file_contents.chunks(chunk_size) {
        send_image_chunk(
            socket,
            chunk, // Current chunk
            chunk_index,
            client_ip,
            client_port,
        )
        .await?;

        chunk_index += 1;
    }


    // Step 3: Send "END" marker
    let dest = (client_ip, client_port);
    com::send(&socket.socket_server_client_tx, "END".to_string(), dest).await?;
    println!("END marker sent. Image transmission complete.");

    Ok(())
}


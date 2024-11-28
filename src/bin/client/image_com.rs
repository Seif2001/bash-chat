use async_std::path::{Path, PathBuf};
use rand::seq::index;
use tokio::net::UdpSocket;
use crate::config::{self, Config};
use crate::socket::{self, Socket};
use crate::{com, image_processor};
use std::fs::create_dir_all;
use std::net::Ipv4Addr;
use async_std::sync::Arc;
use async_std::stream::StreamExt;
use async_std::fs::{File, read_dir};
use async_std::io::prelude::*;
use std::convert::TryInto;
use std::io::Write;
use tokio::sync::Mutex;
use image_processor::decode_image;
// send to leader image

pub async fn send_images_from_to(
    image_path: &str,
    mut num_images: usize,
    client_order: u32,
    server_ip: Ipv4Addr,
    server_port: u16,
    send_socket: &Socket,
    config: &Config
) -> std::io::Result<()> {

    println!("\n******************************************************************************");
    println!(
        "Client {} is sending {} images from '{}' to addr {}:{}",
        client_order, 
        num_images,
        image_path, 
        server_ip,
        server_port
    );
    println!("********************************************************************************");


    let mut read_dir = read_dir(image_path).await?;
    while let Some(entry) = read_dir.next().await {
        if num_images == 0 {
            break;
        }
        num_images -= 1;

        let entry = entry?;
        let path = entry.path();

        if path.is_file().await {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                
                println!(" \n >>>>>>>>>>>>>>>> file: {} <<<<<<<<<<<<<<<", file_name);
                
                // Send the "START" message directly to the server
                // let dest = (server_ip, server_port);
                // com::send(&send_socket.socket_client_server_tx, file_name.to_string(), dest).await?;
                // println!(" --  'Name' message sent ");

                // // Receive leader's address from the server
                // let (ack, _) = com::recv(&send_socket.socket_client_rx).await?;
                // println!(" --  received name ack to begin sending: {}", ack); // already inside the receive leader function

                send_image(&send_socket, file_name, server_ip, server_port, 1020, &config).await?;


                // Receive image
                receive_image(&send_socket, config).await?;
            }
        }
    }

    println!("\nClient {} completed sending images to addr {}.", client_order, server_ip);
    println!("----------------------------------------------------------------------------");
    println!("----------------------------------------------------------------------------\n");

    Ok(())
}
//Receiving
pub async fn recv_image_name(socket: &Socket, config: &Config) -> Result<(PathBuf, Ipv4Addr), std::io::Error> {
    //let socket_client_server_rx = &socket.socket_client_rx;
    let socket_client_server_tx = socket.new_client_socket().await;
    let (image_name, src) = com::recv(&socket.socket_client_rx).await?;
    let image_name = image_name.trim();
    println!("Received image name: {}", image_name);

    let image_path = Path::new(&config.client_encoded_images_dir).join(&image_name);
    println!("Image path: {:?}", image_path);
    let server_ip = match src {
        std::net::SocketAddr::V4(addr) => *addr.ip(),
        std::net::SocketAddr::V6(addr) => {
            panic!("Expected Ipv4Addr but got Ipv6Addr: {}", addr)
        }
    };
    

    let dest = (
        src.ip()
            .to_string()
            .parse::<Ipv4Addr>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid IP address"))?,
        src.port(),
    );

    com::send(&socket_client_server_tx, "NAME_ACK".to_string(), dest).await?;
    println!("NAME_ACK sent.");
    Ok((image_path.to_path_buf(), server_ip))
}

pub async fn recv_image_chunk(
    socket: &Socket,
    config: &Config,
    expected_chunk_index: u32,
    image_data: Arc<Mutex<Vec<u8>>>,
) -> Result<Option<u32>, std::io::Error> {
    let socket_client_rx = socket.socket_client_rx.clone();
    let socket_client_tx = socket.new_client_socket().await;

    let (buf, src) = com::recv_raw(&socket_client_rx).await?;
    let len = buf.len();
    println!("Received chunk of size {} bytes.", len);
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
        com::send(&socket_client_tx, chunk_index.to_string(), dest)
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
    let (image_path, server_ip) = recv_image_name(socket, config).await?;
    println!("Image name received: {:?}", image_path);

    let image_data = Arc::new(Mutex::new(Vec::new()));
    let mut expected_chunk_index = 0;

    loop {
        match recv_image_chunk(socket, config, expected_chunk_index, image_data.clone()).await? {
            Some(next_chunk_index) => {
                expected_chunk_index = next_chunk_index;
            }
            None => {
                println!("Saving received image...");
                let mut file = File::create(&image_path).await?;
                file.write_all(&image_data.lock().await).await?;
                println!("Image saved at: {:?}", image_path);
                file.flush().await?;
                decode_received_image(image_path.to_str().unwrap());
                break;
            }
        }
    }

    Ok(())
}

fn decode_received_image(encoded_image_path: &str) {
    let decoded_dir = Path::new(&Config::new().client_decoded_images_dir).to_str().unwrap().to_string();
    create_dir_all(decoded_dir.clone()).expect("Failed to create decoded images directory");
    
    // Define the output image path in the decoded images directory
    let output_image_path = format!("{}/{}", decoded_dir, encoded_image_path.split('/').last().unwrap());

    println!(" ---> Decoding received image...");

      // Call `decode_image` and log success after the call.
    decode_image(encoded_image_path.to_string(), output_image_path.to_string());
    println!(" --  Image successfully decoded and saved as '{}'", output_image_path);
}

//Sending
pub async fn send_image_name(
    socket: Arc<Mutex<UdpSocket>>,
    image_name: &str,
    server_ip: Ipv4Addr,
    server_port: u16,
) -> Result<(), std::io::Error> {
    let dest = (server_ip, server_port);
    com::send(&socket, image_name.to_string(), dest).await?;

    let (ack, _) = com::recv(&socket).await?;
    if ack.trim() != "NAME_ACK" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to receive NAME_ACK",
        ));
    }

    println!("Image name '{}' sent and acknowledged.", image_name);
    Ok(())
}

pub async fn send_image_chunk(
    socket: Arc<Mutex<UdpSocket>>,
    chunk_data: &[u8],
    chunk_index: u32,
    server_ip: Ipv4Addr,
    server_port: u16,
) -> Result<(), std::io::Error> {
     // Prepare the chunk index as a byte array
     let mut data_with_index = chunk_index.to_be_bytes().to_vec();
     // Append the chunk data
     data_with_index.extend_from_slice(chunk_data);
     
    let dest = (server_ip, 6376);
    
    println!("size of chunk: {}", data_with_index.len());
    com::send_vec(&socket, data_with_index, dest).await?;
    println!("Chunk {} sent.", chunk_index);
    let (ack, _) = com::recv(&socket).await?;
    let ack_index: u32 = ack
        .trim()
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ACK received"))?;

    if ack_index != chunk_index {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Chunk index mismatch: expected {}, got {}",
                chunk_index, ack_index
            ),
        ));
    }

    println!("Chunk {} sent and acknowledged.", chunk_index);
    Ok(())
}
pub async fn send_image(
    socket: &Socket,
    image_name: &str,
    server_ip: Ipv4Addr,
    server_port: u16,
    chunk_size: usize,
    config: &Config
) -> Result<(), std::io::Error> {
    let image_path = format!("./src/bin/client/raw_images/{}", image_name);
    let socket = socket.new_client_socket().await;
    let socket_clone = socket.clone();
    send_image_name(socket, &image_name, server_ip, server_port).await?;
    let mut file = File::open(image_path).await?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).await?;
    println!("File read into memory, size: {} bytes", file_contents.len());

    let mut chunk_index: u32 = 0;

    for chunk in file_contents.chunks(chunk_size) {
        send_image_chunk(
            socket_clone.clone(),
            chunk, // Current chunk
            chunk_index,
            server_ip,
            server_port,
        )
        .await?;

        chunk_index += 1;
    }
    
    // Send an "END" marker to signal the completion of the image transmission
    let dest = (server_ip, config.port_server_client_rx);
    com::send(&socket_clone, "END".to_string(), dest).await?;
    println!("END marker sent. Image transmission complete.");

    Ok(())
}

pub async fn recv_image_client(
    socket: &Socket,
    config: &Config,
) -> Result<(), std::io::Error> {
    let (image_path, server_ip) = recv_image_name(socket, config).await?;
    println!("Image name received: {:?}", image_path);

    let image_data = Arc::new(Mutex::new(Vec::new()));
    let mut expected_chunk_index = 0;

    loop {
        match recv_image_chunk(socket, config, expected_chunk_index, image_data.clone()).await? {
            Some(next_chunk_index) => {
                expected_chunk_index = next_chunk_index;
            }
            None => {
                println!("Saving received image...");
                let mut file = File::create(&image_path).await?;
                file.write_all(&image_data.lock().await).await?;
                println!("Image saved at: {:?}", image_path);
                file.flush().await?;
                decode_received_image(image_path.to_str().unwrap());
                break;
            }
        }
    }

    Ok(())
}


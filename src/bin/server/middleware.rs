use tokio::net::UdpSocket;
use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
use tokio::sync::Mutex; // Using Tokio's async Mutex
use std::sync::Arc;
use std::net::{Ipv4Addr};
use std::io; 
use std::fs::File;

// now for image encryption 
extern crate steganography;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;
use image::{DynamicImage, RgbaImage, Rgba,ImageBuffer, GenericImageView};
use std::fs::metadata;
use std::io::Read;
use std::cmp::min;
use std::io::Write;
use crate::image_processor;


type Db = Arc<Mutex<HashMap<String, String>>>;

pub async fn process(socket: Arc<UdpSocket>, db: Db, data: Vec<u8>, addr: std::net::SocketAddr) {

    let request = String::from_utf8(data).unwrap();

    // Split the request into command and key/value
    let parts: Vec<&str> = request.trim().split_whitespace().collect();

    let response = if parts.len() < 1 {
        "ERROR: Invalid command".to_string()
    } else {
        let command = parts[0].to_uppercase();

        match command.as_str() {
            "SET" if parts.len() == 3 => {
                let key = parts[1].to_string();
                let value = parts[2].to_string();
                // let mut db = db.lock().unwrap();
                let mut db = db.lock().await;
                db.insert(key, value);
                "OK".to_string()
            }
            "GET" if parts.len() == 2 => {
                let key = parts[1];
                // let db = db.lock().unwrap();
                let db = db.lock().await; // Await async lock directly without `.unwrap()`

                if let Some(value) = db.get(key) {
                    value.clone()
                } else {
                    "ERROR: Key not found".to_string()
                }
            }
            _ => "ERROR: Unknown command".to_string(),
        }
    };

    // Send the response back to the client
    socket.send_to(response.as_bytes(), &addr).await.unwrap();
}


pub async fn join(socket: Arc<UdpSocket>, interface_addr:Ipv4Addr, multicast_addr:Ipv4Addr ) -> std::io::Result<()> {

    socket.join_multicast_v4(multicast_addr,interface_addr)?;

    println!("Joined multicast group on interface: {}", interface_addr);
    Ok(())
}

pub async fn send_rpc(
    socket: Arc<UdpSocket>,
    multicast_addr: Ipv4Addr,
    port: u16,
    message: String,
) ->std::io::Result<()>{
    let multicast_socket = (multicast_addr, port);
    // Send the message
    socket.send_to(message.as_bytes(), multicast_socket).await?;

    println!("Message sent to multicast group at {}:{}", multicast_addr, port);
    Ok(())
   
}

pub async fn recv_rpc(socket: Arc<UdpSocket>) {
    let mut buf = vec![0u8; 1024];
    let Ok((len, addr)) = socket.recv_from(&mut buf).await else{return};
    let data = buf[..len].to_vec();
    let message = String::from_utf8(data).unwrap();

}

// pub async fn start_receive_task(socket: Arc<UdpSocket>, image_data: Arc<Mutex<Vec<u8>>>) {
//     tokio::spawn(async move {
//         receive_image_data(socket, image_data)
//             .await
//             .expect("Failed to receive image data");
//     });
// }


/// Middleware function to receive image data via UDP and save it to a file without encryption
// pub async fn receive_image_data(
//     socket: Arc<UdpSocket>, 
//     image_data: Arc<Mutex<Vec<u8>>>) -> io::Result<()> {
    
//     let mut buf = vec![0u8; 1028]; // 4 bytes for index + 1024 bytes for data

//     loop {
//         // Receive data from the socket
//         let (len, addr) = socket.recv_from(&mut buf).await.expect("Failed to receive data");

//         // Check for the "END" signal to finish transmission
//         if len == 3 && &buf[..len] == b"END" {
//             println!("Received end-of-transmission signal. Saving file...");

//             // Write the accumulated image data to a file
//             let mut file = File::create("received_image.jpg")?;
//             let image_data = image_data.lock().await;
//             file.write_all(&image_data)?;

//             println!("Image saved as 'received_image.jpg'");
//             break;
//         } else {
//             // Extract the chunk index from the first 4 bytes
//             let chunk_index = u32::from_be_bytes(buf[..4].try_into().unwrap());

//             // Lock the image buffer and append the received data
//             let mut image_data = image_data.lock().await;
//             image_data.extend_from_slice(&buf[4..len]);
//             println!("Received {} bytes for chunk {}", len - 4, chunk_index);

//             // Send acknowledgment back to the client for the received chunk
//             socket.send_to(&chunk_index.to_be_bytes(), addr).await?;
//         }
//     }
    
//     Ok(())
// }


// encoding and decoding 
pub fn server_encode_image(input_image: &str, encoded_image: &str, cover_image: &str) {
    image_processor::encode_image(input_image.to_string(), encoded_image.to_string(), cover_image.to_string());
}
pub fn server_decode_image(encoded_image: &str, output_image: &str) {
    image_processor::decode_image(encoded_image.to_string(), output_image.to_string());
}


// pub async fn receive_image_data(
//     socket: Arc<UdpSocket>,
//     image_data: Arc<Mutex<Vec<u8>>>,
//     client_addr: Arc<Mutex<Option<std::net::SocketAddr>>>,
// ) -> io::Result<()> {
//     let mut buf = vec![0u8; 1028]; // 4 bytes for index + 1024 bytes for data

//     loop {
//         let (len, addr) = socket.recv_from(&mut buf).await.expect("Failed to receive data");

//         // Store client address for response
//         *client_addr.lock().await = Some(addr);

//         if len == 3 && &buf[..len] == b"END" {
//             println!("Received end-of-transmission signal. Saving and encoding file...");

//             // Write the accumulated image data to a file
//             let mut file = File::create("received_image.jpg")?;
//             let image_data = image_data.lock().await;
//             file.write_all(&image_data)?;

//             // Encode the image with steganography using the server-side function
//             server_encode_image("received_image.jpg", "encoded_image.jpg", "cover_image.jpg");

//             println!("Image encoded. Sending back to client...");
//             send_encoded_image(socket.clone(), client_addr.clone(), "encoded_image.jpg").await?;
//             println!("Encoded image sent to client.");
//             break;
//         } else {
//             let chunk_index = u32::from_be_bytes(buf[..4].try_into().unwrap());

//             let mut image_data = image_data.lock().await;
//             image_data.extend_from_slice(&buf[4..len]);
//             println!("Received {} bytes for chunk {}", len - 4, chunk_index);

//             socket.send_to(&chunk_index.to_be_bytes(), addr).await?;
//         }
//     }
//     Ok(())
// }
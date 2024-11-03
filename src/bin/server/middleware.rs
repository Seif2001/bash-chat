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

pub async fn send_message(
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



// encoding and decoding 
pub fn server_encode_image(input_image: &str, encoded_image: &str, cover_image: &str) {
    image_processor::encode_image(input_image.to_string(), encoded_image.to_string(), cover_image.to_string());
}
pub fn server_decode_image(encoded_image: &str, output_image: &str) {
    image_processor::decode_image(encoded_image.to_string(), output_image.to_string());
}


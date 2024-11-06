
use core::str;
use std::net::UdpSocket;
use std::fs::{File, read_dir};
use std::io::{self, Read, Write};
use std::time::Duration;
use std::net::{Ipv4Addr};
// use std::io;

pub mod image_processor; // Add this if image_processor.rs is in the same directory
use image_processor::decode_image;




// fn decode_received_image() {
//     let encoded_image_path = "received_encoded_image.png";
//     let output_image_path = "decrypted_image.png";

//     println!("Decoding received image...");

//     // Call `decode_image` and handle success or failure
//     match decode_image(encoded_image_path.to_string(), output_image_path.to_string()) {
//         Ok(_) => println!("Image successfully decoded and saved as '{}'", output_image_path),
//         Err(e) => println!("Failed to decode and save image: {:?}", e),
//     }
// }

fn decode_received_image() {
    let encoded_image_path = "client_received_encoded_image.png"; // Corrected file name
    let output_image_path = "decrypted_image.png";

    println!("Decoding received image...");

    // Call `decode_image` and handle success or failure
    match decode_image(encoded_image_path.to_string(), output_image_path.to_string()) {
        Ok(_) => println!("Image successfully decoded and saved as '{}'", output_image_path),
        Err(e) => println!("Failed to decode and save image: {:?}", e),
    }
}



fn receive_encoded_image(socket: &UdpSocket) -> io::Result<()> {
    let mut buf = [0u8; 1028];
    let mut expected_chunk_index = 0;
    let mut file: Option<File> = None;
    let test_socket = UdpSocket::bind("0.0.0.0:9002")?;

    println!("Waiting to receive encoded image from server on socket: {}...", socket.local_addr()?);

    loop {
        match test_socket.recv_from(&mut buf) {
            Ok((len, server_addr)) => {
                if len == 3 && &buf[..len] == b"END" {
                    println!("End of encoded image transmission received.");
                    if let Some(f) = file {
                        f.sync_all()?; // Ensure all data is written to disk
                        println!("Encoded image successfully saved as 'client_received_encoded_image.png'");
                    } else {
                        println!("No data received.");
                    }
                    break;
                }

                if file.is_none() {
                    file = Some(File::create("client_received_encoded_image.png")?);
                }

                let chunk_index = i32::from_be_bytes(buf[..4].try_into().unwrap());

                if chunk_index == expected_chunk_index {
                    if let Some(f) = file.as_mut() {
                        f.write_all(&buf[4..len])?;
                    }
                    expected_chunk_index += 1;
                    socket.send_to(&chunk_index.to_be_bytes(), server_addr)?;
                } else {
                    println!("Received out-of-order chunk {}", chunk_index);
                }
            }
            Err(e) => {
                println!("Error receiving data: {:?}", e);
                return Err(e);
            }
        }
    }

    decode_received_image();
    Ok(())
}

// Function to send a message to the server 
fn send_start_message(socket: &UdpSocket, server_addr: &str, message: &str) -> io::Result<()> {
    socket.send_to(message.as_bytes(), server_addr)?;
    println!("Message sent to server at {}", server_addr);
    Ok(())
}

fn receive_leader(socket: &UdpSocket) -> io::Result<String> {
    loop {
        let mut buf = [0u8; 1024]; // Buffer for receiving the string data

        // Try to receive data from the socket
        match socket.recv_from(&mut buf) {
            Ok((len, _)) => {
                // Convert the received bytes to a string slice
                if let Ok(message) = str::from_utf8(&buf[..len]) {
                    println!("Received leader address: '{}'", message);
                    // Validate if the message is in the "0.0.0.0:port" format
                    if message.parse::<std::net::SocketAddr>().is_ok() {
                        // Return the received string as-is
                        return Ok(message.to_string());
                    } else {
                        eprintln!("Received invalid address format: '{}'", message);
                    }
                } else {
                    eprintln!("Failed to parse string from received data");
                }
            }
            Err(e) => return Err(e), // Return error if there's an issue with the socket
        }
    }
}

// Function to send an image to the server
fn send_image_to_leader(socket: &UdpSocket, server_addr: &str, image_path: &str) -> io::Result<()>{
    send_start_message(socket, server_addr, "START")?;
    let leader_address = receive_leader(socket)?;
    send_image_to_server(socket, &leader_address, image_path)?;
    Ok(())
}

fn send_image_to_server(socket: &UdpSocket, server_addr: &str, image_path: &str) -> io::Result<()> {

    // fn send_message_to_server(socket: &UdpSocket, server_addr: &str, message: &str) -> io::Result<()> {
    



    let mut file = File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let chunk_size = 1024;
    let total_chunks = (buffer.len() as f64 / chunk_size as f64).ceil() as i32;

    let mut chunk_index: i32 = 0;
    // Send total number of chunks before image data
    let num_chunks_bytes = total_chunks.to_be_bytes();
    // socket.send_to(&num_chunks_bytes, server_addr)?;

    println!("\n\nSending image file of {} bytes in {} chunks...", buffer.len(), total_chunks);
    println!("********************************");
    println!("********************************");

    for chunk in buffer.chunks(chunk_size) {
        loop {
            let mut packet = Vec::with_capacity(chunk.len() + 4);
            packet.extend_from_slice(&chunk_index.to_be_bytes()); // Add the chunk index
            packet.extend_from_slice(chunk);

            socket.send_to(&packet, server_addr)?;

            // Set a timeout for receiving the acknowledgment
            socket.set_read_timeout(Some(Duration::from_millis(500)))?;
            let mut ack_buf = [0u8; 4];

            match socket.recv_from(&mut ack_buf) {
                Ok((4, _)) if ack_buf == chunk_index.to_be_bytes() => {
                    let progress = (chunk_index + 1) as f64 / total_chunks as f64 * 100.0;
                    print!("\rProgress: {:.2}% - Chunk {} acknowledged", progress, chunk_index);
                    io::stdout().flush().unwrap(); // Flush to update the same line
                    break; // Proceed to next chunk
                }
                _ => {
                    println!("Timeout or wrong acknowledgment for chunk {}, retrying...", chunk_index);
                    continue; // Retry the same chunk
                }
            }
        }
        chunk_index += 1;
    }
    // Send an end-of-transmission signal
    socket.send_to(b"END", server_addr)?;
    println!("\nImage sent successfully to {}", server_addr);
    println!("********************************");
    println!("********************************\n\n");

    Ok(())
}

// Function to send all images in a directory to the server
fn send_all_images_to_server(socket: &UdpSocket, server_addr: &str, image_path: &str) -> io::Result<()> {
    for entry in read_dir(image_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                println!("\nSending file: {}", file_name);
                send_image_to_server(socket, server_addr, path.to_str().unwrap())?;
                println!("Finished sending file: {}\n", file_name);
            }
        }
    }
    Ok(())
}




fn main() -> io::Result<()> {
    // Separate socket for sending
    let send_socket = UdpSocket::bind("0.0.0.0:9000")?;
    // Get the bound port of send_socket and add 1 to it
    let receive_port = send_socket.local_addr()?.port() + 1;
    
    // Separate socket for receiving, bound to send_socket's port + 1
    let receive_socket = UdpSocket::bind(("0.0.0.0", receive_port))?;

    let server_addr = "10.7.57.111:6274";
    let image_name = "image1.jpg";
    let image_path = "./raw_images/";

    // Send image to the server using the send socket
    send_image_to_leader(&send_socket, server_addr, &format!("{}{}", image_path, image_name))?;
    //send_image_to_server(&send_socket, server_addr, &format!("{}{}", image_path, image_name))?;

    // Listen for the encoded image data from the server using the receive socket
    receive_encoded_image(&receive_socket)?;

    Ok(())
}

use core::str;
use std::net::{UdpSocket, Ipv4Addr, SocketAddr};
use std::fs::{File, read_dir};
use std::io::{self, Read, Write};
use std::time::Duration;
use std::thread;

// use std::net::{Ipv4Addr};
// use std::io;

pub mod image_processor; // Add this if image_processor.rs is in the same directory
use image_processor::decode_image;
use serde_json::ser;




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

    println!(" ---> Decoding received image...");

    // Call `decode_image` and handle success or failure
    match decode_image(encoded_image_path.to_string(), output_image_path.to_string()) {
        Ok(_) => println!(" --  Image successfully decoded and saved as '{}'", output_image_path),
        Err(e) => println!(" *** Failed to decode and save image: {:?}", e),
    }
}



fn receive_encoded_image(socket: &UdpSocket) -> io::Result<()> {
    let mut buf = [0u8; 1028];
    let mut expected_chunk_index = 0;
    let mut file: Option<File> = None;

    println!(" ---> Waiting to receive encoded image from server on socket: {}...", socket.local_addr()?);

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, server_addr)) => {
                if len == 3 && &buf[..len] == b"END" {
                    // println!(" --- End of encoded image transmission received.");
                    if let Some(f) = file {
                        f.sync_all()?; // Ensure all data is written to disk
                        println!(" --  Encoded image successfully saved as 'client_received_encoded_image.png'");
                    } else {
                        println!(" *** No data received.");
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
                    println!(" *** Received out-of-order chunk {}", chunk_index);
                }
            }
            Err(e) => {
                println!(" *** Error receiving data: {:?}", e);
                return Err(e);
            }
        }
    }

    decode_received_image();
    Ok(())
}


fn receive_leader(socket: &UdpSocket) -> io::Result<String> {
    loop {
        let mut buf = [0u8; 1024]; // Buffer for receiving the string data
        let test_socket = UdpSocket::bind("0.0.0.0:9002")?;

        // Try to receive data from the socket
        match test_socket.recv_from(&mut buf) {
            Ok((len, _)) => {
                // Convert the received bytes to a string slice
                if let Ok(message) = str::from_utf8(&buf[..len]) {
                    // println!("Received leader address: '{}'", message);
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

    println!(" ---> Sending image file of {} bytes in {} chunks...", buffer.len(), total_chunks);

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
                    print!("\r --  Progress: {:.2}% - Chunk {} acknowledged", progress, chunk_index);
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
    println!("\n --  Image sent successfully to {}", server_addr);


    Ok(())
}



// Separate thread to constantly listen for encoded images from the server
// fn start_receiving_images(socket: UdpSocket) {
//     thread::spawn(move || {
//         loop {
//             if let Err(e) = receive_encoded_image(&socket) {
//                 eprintln!(" --  Error while receiving encoded image: {:?}", e);
//             }
//         }
//     });
// }


// Function to create multiple clients with specified IP and port configuration
fn create_clients(
    base_ip: Ipv4Addr,
    send_port: u16,
    receive_port: u16,
    num_clients: u32,
    server_addr: &str,
) -> io::Result<Vec<(u32, UdpSocket, UdpSocket)>> {
    let mut clients = Vec::new();

    println!("\n**********************************************************");
    println!("******************* Initializing Clients *****************");
    
    for client_order in 0..num_clients {
        let client_ip = Ipv4Addr::new(
            base_ip.octets()[0],
            base_ip.octets()[1],
            base_ip.octets()[2],
            base_ip.octets()[3] + client_order as u8,
        );

        let send_socket = UdpSocket::bind(SocketAddr::new(client_ip.into(), send_port))?;
        let receive_socket = UdpSocket::bind(SocketAddr::new(client_ip.into(), receive_port))?;

        println!(
            "Created Client {} --> IP: {} \n --  Sending on socket  : {} \n --  Receiving on socket: {}",
            client_order,
            client_ip,
            send_socket.local_addr()?,
            receive_socket.local_addr()?
        );

        // Send "I am awake" message to server
        let awake_message = format!("Client {} is awake", client_order);
        send_socket.send_to(awake_message.as_bytes(), server_addr)?;
        println!("Client {} sent awake message to server at {}", client_order, server_addr);


        // Start a background thread to listen for encoded images on the receive_socket
        let receive_socket_clone = receive_socket.try_clone()?;
        thread::spawn(move || {
            loop {
                if let Err(e) = receive_encoded_image(&receive_socket_clone) {
                    eprintln!("Error while receiving encoded image for client {}: {:?}", client_order, e);
                }
            }
        });

        clients.push((client_order, send_socket, receive_socket));
    }
    
    println!("-----------------------------End--------------------------");
    println!("----------------------------------------------------------\n");
    Ok(clients)
}






// Function to send images from a specified client to a target address
fn send_images_from_to(
    image_path: &str,
    mut num_images: usize,
    client_order: u32,
    server_addr: &str,
    send_socket: &UdpSocket,
    receive_socket: &UdpSocket,
) -> io::Result<()> {

    println!("\n******************************************************************************");
    println!(
        "Client {} is sending {} images from '{}' to addr {}",
        client_order, 
        num_images,
        image_path, 
        server_addr
    );
    println!("********************************************************************************");


    for entry in read_dir(image_path)? {
        if num_images == 0 {
            break;
        }
        num_images -= 1;

        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                
                println!(" \n >>>>>>>>>>>>>>>> file: {} <<<<<<<<<<<<<<<", file_name);
                
                // Send the "START" message directly to the server
                send_socket.send_to(b"START", server_addr)?;
                println!(" --  'START' message sent ");

                // Receive leader's address from the server
                let leader_address = receive_leader(send_socket)?;
                println!(" --  received leader address: {}", leader_address); // already inside the receive leader function

                // Send the image to the server at the leader's address
                send_image_to_server(send_socket, &leader_address, &format!("{}{}", image_path, file_name))?;


                // Wait for acknowledgment from the server
                // receive_encoded_image(receive_socket)?;
            }
        }
    }

    println!("\nClient {} completed sending images to addr {}.", client_order, server_addr);
    println!("----------------------------------------------------------------------------");
    println!("----------------------------------------------------------------------------\n");

    Ok(())
}




fn main() -> io::Result<()> {
    let client_base_ip = Ipv4Addr::new(127, 0, 0, 1);
    let client_server_send_port = 9000;
    let client_server_receive_port = 9001;
    // let client_client_send_port = 9002;
    // let client_client_receive_port = 9003;
    let server_addr = "10.7.57.111:6274";
    let num_clients = 3;

    // Initialize clients and send awake messages to the server
    let clients = create_clients( client_base_ip, 
                                                                    client_server_send_port, 
                                                                    client_server_receive_port, 
                                                                    num_clients, 
                                                                    server_addr)?;

    // Decide which clients should send images
    for (client_order, client_server_send_socket, client_server_receive_socket) in &clients {
        if *client_order == 0 { // Only client number 0 sends images
            let num_images_to_send = 3;
            send_images_from_to("./raw_images/", 
                                num_images_to_send, 
                                *client_order, 
                                server_addr, 
                                client_server_send_socket, 
                                client_server_receive_socket)?;
        }
    }

    Ok(())
}

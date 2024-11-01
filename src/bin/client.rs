use std::net::UdpSocket;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Create a UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?; // Bind to any available local address
    let server_addr = "172.20.10.2:6379"; // Server address

    println!("UDP client is running. Type 'SET <key> <value>' or 'GET <key>' to interact.");

    loop {
        // Read user input
        let mut input = String::new();
        print!("> ");
        io::stdout().flush()?; // Flush stdout to ensure prompt appears
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            continue; // Skip empty input
        }

        // Send the command to the server
        socket.send_to(input.as_bytes(), server_addr)?;

        // Prepare a buffer to receive the response
        let mut buf = [0; 1024];
        let (len, _addr) = socket.recv_from(&mut buf)?;
        let response = String::from_utf8_lossy(&buf[..len]); // Convert response to a String

        // Print the server's response
        println!("Response: {}", response);
    }
}

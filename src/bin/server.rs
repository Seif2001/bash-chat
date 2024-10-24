use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    // Bind a UDP socket to port 6379
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:6379").await.unwrap());
    
    println!("Listening for UDP packets on 127.0.0.1:6379");

    // Shared database
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // Spawn a task to handle receiving and processing
        let socket = socket.clone();
        let db = db.clone();

        // Spawn an asynchronous task for each received message
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            // Asynchronously receive data from the socket
            let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
            let data = buf[..len].to_vec();

            // Process the received data
            process(socket, db, data, addr).await;
        });
    }
}

async fn process(socket: Arc<UdpSocket>, db: Db, data: Vec<u8>, addr: std::net::SocketAddr) {
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
                let mut db = db.lock().unwrap();
                db.insert(key, value);
                "OK".to_string()
            }
            "GET" if parts.len() == 2 => {
                let key = parts[1];
                let db = db.lock().unwrap();
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

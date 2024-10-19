use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:4000").unwrap();

    println!("Listening on {}", socket.local_addr().unwrap());

    let mut buff = [0; 1024];

    loop {
        let (size, source) = socket.recv_from(&mut buff).unwrap();

        let request = String::from_utf8_lossy(&buff[..size]);

        println!("Recieved {} from {}", request, source);

        let response = request;

        socket.send_to(response.as_bytes(), source).unwrap();
    }
}

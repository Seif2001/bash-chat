
use tokio::{net::UdpSocket, sync::Mutex};
use std::sync::Arc;


pub struct Socket{
    pub socket_server_1: Arc<Mutex<UdpSocket>>,
    pub socket_server_2: Arc<Mutex<UdpSocket>>,
    pub socket_server_3: Arc<Mutex<UdpSocket>>,
}

impl Socket{
    pub async fn new(address_server_1:String, address_server_2: String, address_server_3: String) -> Self{
        

        //bind the sockets
        println!("Binding sockets");    
        let socket_server_1 = Arc::new(Mutex::new(UdpSocket::bind(address_server_1).await.expect("Error binding")));
        let socket_server_2 = Arc::new(Mutex::new(UdpSocket::bind(address_server_2).await.expect("Error binding")));
        let socket_server_3 = Arc::new(Mutex::new(UdpSocket::bind(address_server_3).await.expect("Error binding")));
        
        Socket{
            socket_server_1,
            socket_server_2,
            socket_server_3,
        }
    }


}
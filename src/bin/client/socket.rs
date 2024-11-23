
use tokio::{net::UdpSocket, sync::Mutex};
use std::sync::Arc;

use crate::socket;


pub struct Socket{
    pub socket_server_1: Arc<Mutex<UdpSocket>>,
    pub socket_server_2: Arc<Mutex<UdpSocket>>,
    pub socket_server_3: Arc<Mutex<UdpSocket>>,

    pub socket_client_leader_rx: Arc<Mutex<UdpSocket>>,
    pub socket_client_tx: Arc<Mutex<UdpSocket>>,
    pub socket_client_rx: Arc<Mutex<UdpSocket>>,
    pub socket_client_server_tx: Arc<Mutex<UdpSocket>>,
    pub socket_client_dos_tx: Arc<Mutex<UdpSocket>>,
    pub socket_client_dos_rx: Arc<Mutex<UdpSocket>>,

}

impl Socket{
    pub async fn new(address_server_1:String, address_server_2: String, address_server_3: String, client_address_leader_rx: String, client_address_tx: String, client_address_rx: String, address_client_server_tx: String, address_client_dos_tx:String,address_client_dos_rx:String) -> Self{
        

        //bind the sockets
        println!("Binding sockets");    
        let socket_server_1 = Arc::new(Mutex::new(UdpSocket::bind(address_server_1).await.expect("Error binding")));
        let socket_server_2 = Arc::new(Mutex::new(UdpSocket::bind(address_server_2).await.expect("Error binding")));
        let socket_server_3 = Arc::new(Mutex::new(UdpSocket::bind(address_server_3).await.expect("Error binding")));
        let socket_client_leader_rx = Arc::new(Mutex::new(UdpSocket::bind(client_address_leader_rx).await.expect("Error binding")));
        let socket_client_tx = Arc::new(Mutex::new(UdpSocket::bind(client_address_tx).await.expect("Error binding")));
        let socket_client_rx = Arc::new(Mutex::new(UdpSocket::bind(client_address_rx).await.expect("Error binding")));
        let socket_client_server_tx = Arc::new(Mutex::new(UdpSocket::bind(address_client_server_tx).await.expect("Error binding")));
        let socket_client_dos_tx = Arc::new(Mutex::new(UdpSocket::bind(address_client_dos_tx).await.expect("Error binding")));
        let socket_client_dos_rx = Arc::new(Mutex::new(UdpSocket::bind(address_client_dos_rx).await.expect("Error binding")));


        Socket{
            socket_server_1,
            socket_server_2,
            socket_server_3,
            socket_client_leader_rx,
            socket_client_tx,
            socket_client_rx,
            socket_client_server_tx,
            socket_client_dos_tx,
            socket_client_dos_rx
        }
    }


}

use tokio::{net::UdpSocket, sync::Mutex};
use std::sync::Arc;

use crate::socket;

pub struct Socket{
    pub socket_election_tx: Arc<Mutex<UdpSocket>>,
    pub socket_election_rx: Arc<Mutex<UdpSocket>>,
    pub socket_failover_tx: Arc<Mutex<UdpSocket>>,
    pub socket_failover_rx: Arc<Mutex<UdpSocket>>,
    pub socket_client: Arc<Mutex<UdpSocket>>
}

impl Socket{
    pub async fn new(address_election_tx:String, address_election_rx: String, address_failover_tx: String, address_failover_rx:String, address_client: String) -> Self{
        

        //bind the sockets
        println!("Binding sockets");    
        let socket_election_tx = Arc::new(Mutex::new(UdpSocket::bind(address_election_tx).await.expect("Error binding")));
        let socket_election_rx = Arc::new(Mutex::new(UdpSocket::bind(address_election_rx).await.expect("Error binding")));
        let socket_failover_tx = Arc::new(Mutex::new(UdpSocket::bind(address_failover_tx).await.expect("Error binding")));
        let socket_failover_rx = Arc::new(Mutex::new(UdpSocket::bind(address_failover_rx).await.expect("Error binding")));
        let socket_client = Arc::new(Mutex::new(UdpSocket::bind(address_client).await.expect("Error binding")));
        

        Socket{
            socket_client,
            socket_election_tx,
            socket_election_rx,
            socket_failover_tx,
            socket_failover_rx,
        }
    }


}
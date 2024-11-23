
use tokio::{net::UdpSocket, sync::Mutex};
use std::sync::Arc;

use crate::{config::Config, socket};

pub struct Socket{
    pub socket_election_tx: Arc<Mutex<UdpSocket>>,
    pub socket_election_rx: Arc<Mutex<UdpSocket>>,
    pub socket_failover_tx: Arc<Mutex<UdpSocket>>,
    pub socket_failover_rx: Arc<Mutex<UdpSocket>>,
    pub socket_client_elections_rx: Arc<Mutex<UdpSocket>>,
    pub socket_client_leader_tx: Arc<Mutex<UdpSocket>>,
    pub socket_server_client_rx: Arc<Mutex<UdpSocket>>,
    pub socket_server_client_tx: Arc<Mutex<UdpSocket>>,
    pub socket_client_dos_tx: Arc<Mutex<UdpSocket>>,
    pub socket_client_dos_rx: Arc<Mutex<UdpSocket>>,

    pub socket_server_dos_rx: Arc<Mutex<UdpSocket>>

}

impl Socket{
    pub async fn new(config:Config) -> Self{
        

        //bind the sockets
        println!("Binding sockets");    
        let socket_election_tx = Arc::new(Mutex::new(UdpSocket::bind(config.address_election_tx).await.expect("Error binding")));
        let socket_election_rx = Arc::new(Mutex::new(UdpSocket::bind(config.address_election_rx).await.expect("Error binding")));
        let socket_failover_tx = Arc::new(Mutex::new(UdpSocket::bind(config.address_failover_tx).await.expect("Error binding")));
        let socket_failover_rx = Arc::new(Mutex::new(UdpSocket::bind(config.address_failover_rx).await.expect("Error binding")));
        let socket_client_elections_rx = Arc::new(Mutex::new(UdpSocket::bind(config.address_client_elections_rx).await.expect("Error binding")));
        let socket_client_leader_tx = Arc::new(Mutex::new(UdpSocket::bind(config.address_client_leader_tx).await.expect("Error binding")));
        let socket_server_client_rx = Arc::new(Mutex::new(UdpSocket::bind(config.address_server_client_rx).await.expect("Error binding")));
        let socket_server_client_tx = Arc::new(Mutex::new(UdpSocket::bind(config.address_server_client_tx).await.expect("Error binding")));
        let socket_client_dos_tx = Arc::new(Mutex::new(UdpSocket::bind(config.address_client_dos_tx).await.expect("Error binding")));
        let socket_client_dos_rx = Arc::new(Mutex::new(UdpSocket::bind(config.address_client_dos_rx).await.expect("Error binding")));

        let socket_server_dos_rx = Arc::new(Mutex::new(UdpSocket::bind(config.address_server_dos_rx).await.expect("Error binding")));


        

        Socket{
            socket_client_elections_rx,
            socket_election_tx,
            socket_election_rx,
            socket_failover_tx,
            socket_failover_rx,
            socket_client_leader_tx,
            socket_server_client_rx,
            socket_server_client_tx,
            socket_client_dos_tx,
            socket_client_dos_rx,
            socket_server_dos_rx
        }
    }


}
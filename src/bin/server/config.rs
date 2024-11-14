use std::env;
use dotenv::dotenv;
use std::net::Ipv4Addr;
use std::str::FromStr;



#[derive(Debug)]
pub struct Config{
    pub address_election_tx: String,
    pub address_election_rx: String,
    pub address_failover_tx: String,
    pub address_failover_rx: String,
    pub multicast_addr: Ipv4Addr,
    pub interface_addr: Ipv4Addr,
    pub port_election_tx: u16,
    pub port_election_rx: u16,

}

impl Config {
    pub fn new() -> Config{
        dotenv().ok();
        
        let port_election_tx = env::var("PORT_ELECTION_TX").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_election_rx = env::var("PORT_ELECTION_RX").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        
        let port_bully_tx = env::var("PORT_BULLY_TX").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
        let port_bully_rx = env::var("PORT_BULLY_RX").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
       
    


        // Define server addresses
        let server_ip = "0.0.0.0:";
        
        let address_election_tx = format!("{}{}", server_ip, port_election_tx);
        let address_election_rx = format!("{}{}", server_ip, port_election_rx);
        let address_failover_tx = format!("{}{}", server_ip, port_bully_tx);
        let address_failover_rx = format!("{}{}", server_ip, port_bully_rx);

        

        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");

        Config {
            address_election_tx,
            address_election_rx,
            address_failover_tx,
            address_failover_rx,
            multicast_addr,
            interface_addr,
            port_election_tx,
            port_election_rx,
        }
    }
    pub fn get_address_election_tx(&self) -> String{
        self.address_election_tx.clone()
    }
    pub fn get_address_election_rx(&self) -> String{
        self.address_election_rx.clone()
    }
    pub fn get_address_failover_tx(&self) -> String{
        self.address_failover_tx.clone()
    }
    pub fn get_address_failover_rx(&self) -> String{
        self.address_failover_rx.clone()
    }
    pub fn get_multicast_addr(&self) -> Ipv4Addr{
        self.multicast_addr.clone()
    }
    pub fn get_interface_addr(&self) -> Ipv4Addr{
        self.interface_addr.clone()
    }
    pub fn get_port_election_tx(&self) -> u16{
        self.port_election_tx.clone()
    }
    pub fn get_port_election_rx(&self) -> u16{
        self.port_election_rx.clone()
    }
    
}
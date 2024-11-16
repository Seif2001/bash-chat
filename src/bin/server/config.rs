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
    pub address_client: String,
    pub multicast_addr: Ipv4Addr,
    pub interface_addr: Ipv4Addr,
    pub port_election_tx: u16,
    pub port_election_rx: u16,
    pub port_failover_tx: u16,
    pub port_client: u16,

}

impl Config {
    pub fn new() -> Config{
        dotenv().ok();
        
        let port_election_tx = env::var("PORT_ELECTION_TX").expect("PORT_ELECTION_TX not set").parse::<u16>().expect("Invalid server port");
        let port_election_rx = env::var("PORT_ELECTION_RX").expect("PORT_ELECTION_RX not set").parse::<u16>().expect("Invalid server port");
        
        let port_failover_tx = env::var("PORT_FAILOVER_TX").expect("PORT_FAILOVER_TX not set").parse::<u16>().expect("Invalid failovertx port");
        let port_bully_rx = env::var("PORT_BULLY_RX").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
       
        let port_client = env::var("PORT_CLIENT").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");




        // Define server addresses
        let server_ip = "0.0.0.0:";
        
        let address_election_tx = format!("{}{}", server_ip, port_election_tx);
        let address_election_rx = format!("{}{}", server_ip, port_election_rx);
        let address_failover_tx = format!("{}{}", server_ip, port_failover_tx);
        let address_failover_rx = format!("{}{}", server_ip, port_bully_rx);
        let address_client = format!("{}{}", server_ip, port_client);

        

        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");

        Config {
            address_client,
            address_election_tx,
            address_election_rx,
            address_failover_tx,
            address_failover_rx,
            multicast_addr,
            interface_addr,
            port_election_tx,
            port_election_rx,
            port_failover_tx,
            port_client
        }
    }
    
}
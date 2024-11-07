use std::env;
use dotenv::dotenv;
use std::net::Ipv4Addr;

#[derive(debug)]
struct Config{
    address_election_tx: String,
    address_election_rx: String,
    address_bully_tx: String,
    address_bully_rx: String,
    multicast_addr: Ipv4Addr,
    interface_addr: Ipv4Addr
}

impl Config {
    pub fn new() -> Config{
        dotenv.ok();
        
        let port_election_tx = env::var("PORT_ELECTION_TX").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        let port_election_rx = env::var("PORT_ELECTION_RX").expect("PORT_SERVER not set").parse::<u16>().expect("Invalid server port");
        
        let port_bully_tx = env::var("PORT_BULLY_TX").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
        let port_bully_rx = env::var("PORT_BULLY_RX").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
       
    


        // Define server addresses
        let server_ip = "0.0.0.0:";
        
        let address_election_tx = format!("{}{}", server_ip, port_election_tx);
        let address_election_rx = format!("{}{}", server_ip, port_election_rx);
        let address_bully_tx = format!("{}{}", server_ip, port_bully_tx);
        let address_bully_rx = format!("{}{}", server_ip, port_bully_rx);

        

        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");

        Config {
            address_election_tx,
            address_election_rx,
            address_bully_tx,
            address_bully_rx,
            multicast_addr,
            interface_addr,
        }
    }
}
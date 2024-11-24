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
    pub port_failover_tx: u16,
    pub port_server_client_rx: u16,
    pub port_server_client_tx: u16,
    pub port_client_dos_tx: u16,
    pub port_client_dos_rx: u16,

    pub port_dos_election_tx:u16,
    pub port_dos_election_rx:u16,

    pub port_server_dos_rx: u16,
    pub port_client_elections_tx: u16,
    pub address_client_elections_rx: String,
    pub port_client_tx: u16,
    pub address_client_leader_tx: String,
    pub port_client_tx_leader: u16,
    pub address_server_client_rx: String,

    pub server_raw_images_dir: String,
    pub server_encoded_images_dir: String,

    pub address_server_client_tx: String,
    pub address_client_dos_tx: String,
    pub address_client_dos_rx: String,
    pub address_dos_election_tx: String,
    pub address_dos_election_rx: String,

    pub address_server_dos_rx: String,

}

impl Config {
    pub fn new() -> Config{
        dotenv().ok();
        
        let port_election_tx = env::var("PORT_ELECTION_TX").expect("PORT_ELECTION_TX not set").parse::<u16>().expect("Invalid server port");
        let port_election_rx = env::var("PORT_ELECTION_RX").expect("PORT_ELECTION_RX not set").parse::<u16>().expect("Invalid server port");
        
        let port_failover_tx = env::var("PORT_FAILOVER_TX").expect("PORT_FAILOVER_TX not set").parse::<u16>().expect("Invalid failovertx port");
        let port_bully_rx = env::var("PORT_BULLY_RX").expect("PORT_BULLY not set").parse::<u16>().expect("Invalid bully port");
        let port_client_tx = env::var("PORT_CLIENT_TX").expect("PORT_CLIENT_TX not set").parse::<u16>().expect("Invalid client port");

        let port_client_elections_rx = env::var("PORT_CLIENT_ELECTIONS_RX").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");
        let port_client_elections_tx = env::var("PORT_CLIENT_ELECTIONS_TX").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");

        let port_client_leader_tx = env::var("PORT_CLIENT_LEADER_TX").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");
        let port_client_tx_leader = env::var("PORT_CLIENT_TX_LEADER").expect("PORT_CLIENT not set").parse::<u16>().expect("Invalid client port");

        let port_server_client_rx = env::var("PORT_SERVER_CLIENT_RX").expect("PORT_SERVER_RX not set").parse::<u16>().expect("Invalid server port");
        let port_server_client_tx = env::var("PORT_SERVER_CLIENT_TX").expect("PORT_SERVER_TX not set").parse::<u16>().expect("Invalid server port");

        let port_client_dos_tx = env::var("PORT_CLIENT_DOS_TX").expect("PORT_CLIENT_DOS_TX not set").parse::<u16>().expect("Invalid server port");
        let port_client_dos_rx = env::var("PORT_CLIENT_DOS_RX").expect("PORT_CLIENT_DOS_RX not set").parse::<u16>().expect("Invalid server port");

        let port_dos_election_tx = env::var("PORT_DOS_ELECTION_TX").expect("PORT_DOS_ELECTION_TX not set").parse::<u16>().expect("Invalid server port");
        let port_dos_election_rx = env::var("PORT_DOS_ELECTION_RX").expect("PORT_DOS_ELECTION_RX not set").parse::<u16>().expect("Invalid server port");

        let port_server_dos_rx = env::var("PORT_SERVER_DOS_RX").expect("PORT_SERVER_DOS_RX not set").parse::<u16>().expect("Invalid server port");

        let server_raw_images_dir = env::var("SERVER_RAW_IMAGES_DIR").expect("SERVER_RAW_IMAGES_DIR not set");
        let server_encoded_images_dir = env::var("SERVER_ENCODED_IMAGES_DIR").expect("SERVER_ENCODED_IMAGES_DIR not set");

        // Define server addresses
        let server_ip = "0.0.0.0:";
        
        let address_election_tx = format!("{}{}", server_ip, port_election_tx);
        let address_election_rx = format!("{}{}", server_ip, port_election_rx);
        let address_failover_tx = format!("{}{}", server_ip, port_failover_tx);
        let address_failover_rx = format!("{}{}", server_ip, port_bully_rx);
        let address_client_elections_rx = format!("{}{}", server_ip, port_client_elections_rx);
        let address_client_leader_tx = format!("{}{}", server_ip, port_client_leader_tx);
        let address_server_client_rx = format!("{}{}", server_ip, port_server_client_rx);
        let address_server_client_tx = format!("{}{}", server_ip, port_server_client_tx);
        let address_client_dos_tx = format!("{}{}", server_ip, port_client_dos_tx);
        let address_client_dos_rx = format!("{}{}", server_ip, port_client_dos_rx);
        let address_dos_election_tx = format!("{}{}", server_ip, port_dos_election_tx);
        let address_dos_election_rx = format!("{}{}", server_ip, port_dos_election_rx);

        let address_server_dos_rx = format!("{}{}", server_ip, port_server_dos_rx);



        

        // Retrieve multicast and interface addresses
        let multicast_addr = Ipv4Addr::from_str(&env::var("MULTICAST_ADDRESS").expect("MULTICAST_ADDRESS not set")).expect("Invalid multicast address");
        let interface_addr = Ipv4Addr::from_str(&env::var("INTERFACE_ADDRESS").expect("INTERFACE_ADDRESS not set")).expect("Invalid interface address");

        Config {
            address_client_elections_rx,
            address_election_tx,
            address_election_rx,
            address_failover_tx,
            address_failover_rx,
            multicast_addr,
            interface_addr,
            port_election_tx,
            port_election_rx,
            port_failover_tx,
            port_server_client_rx,
            port_server_client_tx,
            port_client_dos_tx,
            port_client_dos_rx,
            port_dos_election_tx,
            port_dos_election_rx,
            port_server_dos_rx,
            port_client_elections_tx,
            address_client_leader_tx,
            port_client_tx,
            port_client_tx_leader,
            address_server_client_rx,
            server_raw_images_dir,
            server_encoded_images_dir,
            address_server_client_tx,
            address_client_dos_tx,
            address_client_dos_rx,
            address_dos_election_tx,
            address_dos_election_rx,
            address_server_dos_rx
        }
    }
    
}
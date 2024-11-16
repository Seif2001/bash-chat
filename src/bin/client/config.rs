use std::env;
use dotenv::dotenv;
use std::net::Ipv4Addr;
use std::str::FromStr;



#[derive(Debug)]
pub struct Config{
    pub address_server_1: String,
    pub address_server_2: String,
    pub address_server_3: String,
    pub server_ip_1: Ipv4Addr,
    pub server_ip_2: Ipv4Addr,
    pub server_ip_3: Ipv4Addr,
    pub port_send: u16,
}

impl Config {
    pub fn new() -> Config{
        dotenv().ok();
        
        let port_server_1 = env::var("PORT_SERVER_1").expect("PORT_SERVER_1 not set").parse::<u16>().expect("Invalid server port");
        let port_server_2 = env::var("PORT_SERVER_2").expect("PORT_SERVER_2 not set").parse::<u16>().expect("Invalid server port");
        let port_server_3 = env::var("PORT_SERVER_3").expect("PORT_SERVER_3 not set").parse::<u16>().expect("Invalid server port");

        let server_ip = "127.0.0.1:";

        let address_server_1 = format!("{}{}", server_ip, port_server_1);
        let address_server_2 = format!("{}{}", server_ip, port_server_2);
        let address_server_3 = format!("{}{}", server_ip, port_server_3);

        let server_ip_1 = env::var("SERVER_IP_1").expect("SERVER_IP_1 not set").parse::<Ipv4Addr>().expect("Invalid server ip");
        let server_ip_2 = env::var("SERVER_IP_2").expect("SERVER_IP_2 not set").parse::<Ipv4Addr>().expect("Invalid server ip");
        let server_ip_3 = env::var("SERVER_IP_3").expect("SERVER_IP_3 not set").parse::<Ipv4Addr>().expect("Invalid server ip");

        let port_send = env::var("PORT_CLIENT").expect("PORT_SEND not set").parse::<u16>().expect("Invalid port");

       Config{
            address_server_1,
            address_server_2,
            address_server_3,
            server_ip_1,
            server_ip_2,
            server_ip_3,
            port_send,
        }
    }
    
}
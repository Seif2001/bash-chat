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
    pub port_client_dos_tx:u16,
    pub port_client_dos_rx:u16,
    pub port_client_rx: u16,
    pub port_server_client_rx: u16,
    pub port_client_image_request_rx:u16,

    pub address_client_dos_tx:String,
    pub address_client_dos_rx:String,

    pub client_raw_images_dir: String,
    pub client_encoded_images_dir: String,
    pub client_high_quality_receive_dir:String,
    pub client_decoded_images_dir: String,
    pub client_low_quality_images_dir: String,
    pub client_low_quality_receive_dir: String,


    pub address_client_tx: String,
    pub address_client_rx: String,
    pub address_client_image_request_rx: String,

    pub port_client_elections_rx: u16,
    pub address_client_leader_rx: String,
    pub port_server_rx: u16,
    //pub address_client_server_tx: String,

    pub username: String,
}

impl Config {
    pub fn new() -> Config{
        dotenv().ok();
        
        let port_server_1 = env::var("PORT_SERVER_1").expect("PORT_SERVER_1 not set").parse::<u16>().expect("Invalid server port");
        let port_server_2 = env::var("PORT_SERVER_2").expect("PORT_SERVER_2 not set").parse::<u16>().expect("Invalid server port");
        let port_server_3 = env::var("PORT_SERVER_3").expect("PORT_SERVER_3 not set").parse::<u16>().expect("Invalid server port");

       let port_client_leader_rx = env::var("PORT_CLIENT_TX_LEADER").expect("PORT_CLIENT_RX not set").parse::<u16>().expect("Invalid client port");
        let port_client_elections_rx = env::var("PORT_CLIENT_ELECTIONS_RX").expect("PORT_SERVER_RX not set").parse::<u16>().expect("Invalid server port");
        let port_client_tx = env::var("PORT_CLIENT_TX").expect("PORT_CLIENT_TX not set").parse::<u16>().expect("Invalid client port");
        let port_client_rx = env::var("PORT_CLIENT_RX").expect("PORT_CLIENT_RX not set").parse::<u16>().expect("Invalid client port");
        //let port_client_server_tx = env::var("PORT_CLIENT_SERVER_TX").expect("PORT_CLIENT_SERVER_TX not set").parse::<u16>().expect("Invalid client port");
        let port_server_client_rx = env::var("PORT_SERVER_CLIENT_RX").expect("PORT_SERVER_CLIENT_RX not set").parse::<u16>().expect("Invalid server port");
        let port_server_rx = env::var("PORT_SERVER_RX").expect("PORT_SERVER_RX not set").parse::<u16>().expect("Invalid server port");
        let port_client_image_request_rx = env::var("PORT_CLIENT_IMAGE_REQUEST_RX").expect("PORT_CLIENT_IMAGE_REQUEST_RX not set").parse::<u16>().expect("Invalid server port");
        //let port_server_tx = env::var("PORT_SERVER_RX").expect("PORT_SERVER_RX not set").parse::<u16>().expect("Invalid server port");

        

        let server_ip = "0.0.0.0:";

        let address_server_1 = format!("{}{}", server_ip, port_server_1);
        let address_server_2 = format!("{}{}", server_ip, port_server_2);
        let address_server_3 = format!("{}{}", server_ip, port_server_3);
        let address_client_leader_rx = format!("{}{}", server_ip, port_client_leader_rx);
        let address_client_tx = format!("{}{}", server_ip, port_client_tx);
        let address_client_rx = format!("{}{}", server_ip, port_client_rx);
        let address_client_image_request_rx = format!("{}{}", server_ip, port_client_image_request_rx);
        //let address_client_server_tx = format!("{}{}", server_ip, port_client_server_tx);

        let port_client_dos_tx = env::var("PORT_CLIENT_DOS_TX").expect("PORT_CLIENT_DOS_TX not set").parse::<u16>().expect("Invalid server port");
        let port_client_dos_rx = env::var("PORT_CLIENT_DOS_RX").expect("PORT_CLIENT_DOS_RX not set").parse::<u16>().expect("Invalid server port");


        let address_client_dos_tx = format!("{}{}", server_ip, port_client_dos_tx);
        let address_client_dos_rx = format!("{}{}", server_ip, port_client_dos_rx);

        let client_raw_images_dir = env::var("CLIENT_RAW_IMAGES_DIR").expect("CLIENT_RAW_IMAGES_DIR not set");
        let client_encoded_images_dir = env::var("CLIENT_ENCODED_IMAGES_DIR").expect("CLIENT_ENCODED_IMAGES_DIR not set");
        let client_high_quality_receive_dir = env::var("CLIENT_HIGH_QUALITY_RECEIVE_DIR").expect("CLIENT_HIGH_QUALITY_RECEIVE_DIR not set");

        let client_decoded_images_dir = env::var("CLIENT_DECODED_IMAGES_DIR").expect("CLIENT_DECODED_IMAGES_DIR not set");
        let client_low_quality_images_dir = env::var("CLIENT_LOW_QUALITY_IMAGES_DIR").expect("CLIENT_DECODED_IMAGES_DIR not set");
        let client_low_quality_receive_dir = env::var("CLIENT_LOW_QUALITY_RECEIVE_DIR").expect("CLIENT_LOW_QUALITY_RECEIVE_DIR not set");

        let server_ip_1 = env::var("SERVER_IP_1").expect("SERVER_IP_1 not set").parse::<Ipv4Addr>().expect("Invalid server ip");
        let server_ip_2 = env::var("SERVER_IP_2").expect("SERVER_IP_2 not set").parse::<Ipv4Addr>().expect("Invalid server ip");
        let server_ip_3 = env::var("SERVER_IP_3").expect("SERVER_IP_3 not set").parse::<Ipv4Addr>().expect("Invalid server ip");
        let address_client_image_request_rx = format!("{}{}", server_ip, port_client_image_request_rx);
        let username = env::var("CLIENT_USERNAME").expect("CLIENT_USERNAME not set").parse::<String>().expect("Invalid username");




       Config {
            address_server_1,
            address_server_2,
            address_server_3,
            server_ip_1,
            server_ip_2,
            server_ip_3,
            port_client_dos_tx,
            port_client_dos_rx,
            port_client_rx,
            port_server_client_rx,
            address_client_dos_tx,
            address_client_dos_rx,
            client_raw_images_dir,
            client_encoded_images_dir,
            client_decoded_images_dir,
            client_high_quality_receive_dir,
            client_low_quality_images_dir,
            client_low_quality_receive_dir,
            address_client_tx,
            address_client_rx,
            address_client_image_request_rx,
            port_client_elections_rx,
            address_client_leader_rx,
            port_server_rx,
            port_client_image_request_rx,
            //address_client_server_tx,
            username


        }
    }
    
}
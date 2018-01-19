
use std::thread;
use std::io::{Error};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::collections::HashMap;
use std::string::String;

mod protocol;

pub struct Network {
    num_devices: u32,
    data: HashMap<String, i32>,
    interests: Vec<String>
}

impl Network {

    pub fn new(interests: Vec<String>)-> Network {
        let net: Network = Network{num_devices: 0, interests: interests, data: HashMap::new()};

        net.start_server();
        net.broadcast_info();

        net
    }

    pub fn get_reachable_devices(&self) ->u32 {
        self.num_devices
    }

    fn handle_incoming(&self, res: Result<(TcpStream, SocketAddr), Error>) {
        match res {
            Ok((stream, addr)) => println!("new client: {:?}", addr),
            Err(e) => println!("couldn't get client: {:?}", e),
        }
    }

    pub fn start_server(&self) {

        let listener = TcpListener::bind("127.0.0.1:80").unwrap();


    }

    fn broadcast_info(&self) {

    }


}

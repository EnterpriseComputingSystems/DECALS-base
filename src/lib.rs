
use std::thread;
use std::io::{Error};
use std::net::{TcpListener, TcpStream, SocketAddr};

pub struct Network {connected: bool, num_devices: u32}

impl Network {

    pub fn new()-> Network {
        Network{connected: true, num_devices: 300}
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

        let newConnectionListener = thread::spawn(move || {

            while true {
                self.handle_incoming(listener.accept());
            }
        });
    }


}

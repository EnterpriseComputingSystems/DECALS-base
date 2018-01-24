
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use futures::{Future, Stream};
use tokio_io::{io, AsyncRead};
use tokio_core::net::{TcpListener, UdpSocket};
use tokio_core::reactor::Core;

use std::collections::HashMap;

mod protocol;

pub struct Network {
    num_devices: u32,
    data: HashMap<String, i32>,
    interests: Vec<String>,
    broadcastSock: UdpSocket,

}

impl Network {

    pub fn new(interests: Vec<String>)-> Network {
        let net: Network = Network{num_devices: 0, interests: interests, data: HashMap::new(), broadcastSock: ()};

        net.start_server();
        net.broadcast_info();

        return net;
    }

    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    pub fn start_server(&self) {

        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let addr = "127.0.0.1:0".parse().unwrap();

        let listener = TcpListener::bind(&addr, &handle).unwrap();


    }

    fn broadcast_info(&self) {

    }


}

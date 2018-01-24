
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

        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let udpaddr = "127.0.0.1:52300".parse().unwrap();
        let tcpaddr = "127.0.0.1:0".parse().unwrap();

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        //UDP
        let broadcastSock: UdpSocket = match UdpSocket::bind(&udpaddr, &handle) {
            Ok(sock) => sock,
            Err(error) => {panic!("Couldn't listen for udp");}
        };


        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // TCP
        let listener = match TcpListener::bind(&tcpaddr, &handle) {
            Ok(lst) => lst,
            Err(error) => panic!("Couldn't listen for TCP!")
        };

        let net: Network = Network{num_devices: 0, interests: interests, data: HashMap::new(), broadcastSock: broadcastSock};



        net.broadcast_info();

        return net;
    }

    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    fn broadcast_info(&self) {

    }


}

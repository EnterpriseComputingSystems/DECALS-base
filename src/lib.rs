
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
    broadcast_sock: UdpSocket,
    port: u16,
    tcp_listener: Option<TcpListener>
}

impl Network {

    pub fn new(interests: Vec<String>)-> Network {

        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let udpaddr = "127.0.0.1:52300".parse().unwrap();
        let tcpaddr = "127.0.0.1:0".parse().unwrap();

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        //UDP
        let broadcast_sock: UdpSocket = match UdpSocket::bind(&udpaddr, &handle) {
            Ok(sock) => sock,
            Err(error) => panic!("Couldn't listen for UDP! {}", error)
        };

        broadcast_sock.set_broadcast(true).unwrap();

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // TCP
        let tcp_listener = match TcpListener::bind(&tcpaddr, &handle) {
            Ok(lst) => lst,
            Err(error) => panic!("Couldn't listen for TCP! {}", error)
        };

        let port = tcp_listener.local_addr().unwrap().port();
        println!("TCP Server running on port {}", port);

        let mut net: Network = Network{num_devices: 0,
            interests: interests,
            data: HashMap::new(),
            broadcast_sock: broadcast_sock,
            port: tcp_listener.local_addr().unwrap().port(),
            tcp_listener: Some(tcp_listener)
        };

        net.start();

        return net;
    }

    fn start(&mut self) {

        let tcp_listener = std::mem::replace(&mut self.tcp_listener, None).unwrap();

        let x = tcp_listener.incoming().for_each(|(sk, peer)|{
            println!("ASDASD {}", self.port);
            Ok(())
        });
    }

    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    fn broadcast_info(&self) {
        let addr = "255.255.255.255:52300".parse().unwrap();
        match self.broadcast_sock.send_to(protocol::get_broadcast(self.port, &self.interests).as_bytes(), &addr) {
            Ok(_) => return,
            Err(error) => println!("Error broadcasting {}", error)
        };
    }


}

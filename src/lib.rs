
extern crate futures;
extern crate tokio_io;
#[macro_use]
extern crate tokio_core;

use futures::{Future, Stream, Poll};
use tokio_io::{AsyncRead};
use tokio_core::net::{TcpListener, UdpSocket};
use tokio_core::reactor::Core;

use std::collections::HashMap;
use std::{env, io};

mod protocol;

pub struct UDPServ<'a> {
    net: &'a Network
}

pub struct Network {
    num_devices: u32,
    data: HashMap<String, i32>,
    interests: Vec<String>,
    broadcast_sock: UdpSocket,
    port: u16
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
            port: tcp_listener.local_addr().unwrap().port()
        };


        {
            let net = &mut net;

            let serv = tcp_listener.incoming().for_each(|(sk, peer)|{
                println!("ASDASD {}", net.port);
                Ok(())
            });

            core.run(serv).unwrap();

            let usrv = UDPServ{net: net};

            core.run(usrv).unwrap();
        }

        net.broadcast_info();

        return net;
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

impl<'a> Future for UDPServ<'a> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        let mut buf = vec![0; 1024];
        loop {
            let input = try_nb!(self.net.broadcast_sock.recv_from(&mut buf));

            print!("UDP received {:?}", input);
        }
    }
}


extern crate futures;
extern crate tokio_io;
#[macro_use]
extern crate tokio_core;

use futures::{Future, Stream, Poll};
use tokio_core::net::{TcpListener, UdpSocket};
use tokio_core::reactor::Core;

use std::collections::HashMap;
use std::{io};
use std::thread;
use std::sync::RwLock;
use std::borrow::Borrow;
use std::sync::Arc;

mod protocol;

pub struct UDPServ {
    net: Arc<RwLock<Network>>
}

pub struct Network {
    num_devices: u32,
    data: HashMap<String, i32>,
    interests: Vec<String>,
    broadcast_sock: Option<UdpSocket>,
    port: u16,
    servers_started: bool
}

impl Network {

    pub fn new(interests: Vec<String>)-> Arc<RwLock<Network>> {

        let net: Network = Network{num_devices: 0,
            interests: interests,
            data: HashMap::new(),
            broadcast_sock: None,
            port: 0,
            servers_started: false
        };

        let bx: Arc<RwLock<Network>> = Arc::new(RwLock::new(net));

        Network::start_tcp_serv(bx.clone());
        Network::start_udp_serv(bx.clone());

        {
            let lcktmp: &RwLock<Network> = bx.borrow();
            let guard = lcktmp.read().unwrap();
            (*guard).broadcast_info();
        }

        println!("Servers Started");

        return bx;
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // TCP
    fn start_tcp_serv(network: Arc<RwLock<Network>>) {

        thread::spawn(|| {

            let net: Arc<RwLock<Network>> = network;

            let mut core = Core::new().unwrap();
            let handle = core.handle();

            let tcpaddr = "127.0.0.1:0".parse().unwrap();

            let tcp_listener = match TcpListener::bind(&tcpaddr, &handle) {
                Ok(lst) => lst,
                Err(error) => panic!("Couldn't listen for TCP! {}", error)
            };


            let port = tcp_listener.local_addr().unwrap().port();
            println!("TCP Server running on port {}", port);

            {
                let lcktmp: &RwLock<Network> = net.borrow();
                let mut guard = lcktmp.write().unwrap();
                (*guard).port = port;
            }

            let serv = tcp_listener.incoming().for_each(|(sk, peer)|{
                {
                    let lcktmp: &RwLock<Network> = net.borrow();
                    let guard = lcktmp.read().unwrap();
                    println!("ASDASD {}", (*guard).port);
                }

                Ok(())
            });


            core.run(serv).unwrap();
        });

    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //UDP
    fn start_udp_serv(network: Arc<RwLock<Network>>) {

        thread::spawn(|| {

            let net: Arc<RwLock<Network>> = network;

            let mut core = Core::new().unwrap();
            let handle = core.handle();

            let udpaddr = "127.0.0.1:52300".parse().unwrap();

            let broadcast_sock: UdpSocket = match UdpSocket::bind(&udpaddr, &handle) {
                Ok(sock) => sock,
                Err(error) => panic!("Couldn't listen for UDP! {}", error)
            };

            broadcast_sock.set_broadcast(true).unwrap();

            {
                let lcktmp: &RwLock<Network> = net.borrow();
                let mut guard = lcktmp.write().unwrap();
                (*guard).broadcast_sock = Some(broadcast_sock);
            }

            let usrv = UDPServ{net: net};

            core.run(usrv).unwrap();
        });
    }


    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    fn broadcast_info(&self) {
        let addr = "255.255.255.255:52300".parse().unwrap();

        match self.broadcast_sock.unwrap().send_to(protocol::get_broadcast(self.port, &self.interests).as_bytes(), &addr) {
            Ok(_) => return,
            Err(error) => println!("Error broadcasting {}", error)
        };
    }


}

impl Future for UDPServ {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        let mut buf = vec![0; 1024];
        loop {
            let input;
            {
                let lcktmp: &RwLock<Network> = self.net.borrow();
                let guard = lcktmp.read().unwrap();
                input = try_nb!((*guard).broadcast_sock.unwrap().recv_from(&mut buf));
            }


            print!("UDP received {:?} {:?}", buf, input);
        }
    }
}

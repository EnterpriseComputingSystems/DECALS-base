
extern crate futures;
extern crate tokio_io;
#[macro_use]
extern crate tokio_core;

use futures::{Future, Stream, Poll};
use tokio_core::net::{TcpListener, UdpSocket};
use tokio_core::reactor::Core;

use std::collections::HashMap;
use std::{io, thread, time};
use std::sync::{RwLock, Arc};
use std::borrow::Borrow;

mod protocol;

pub struct UDPServ {
    net: Arc<RwLock<Network>>
}

pub struct Network {
    num_devices: u32,
    data: HashMap<String, i32>,
    interests: Vec<String>,
    broadcast_sock: Option<UdpSocket>,
    port: u16
}

impl Network {

    pub fn new(interests: Vec<String>)-> Arc<RwLock<Network>> {

        let net: Network = Network{num_devices: 0,
            interests: interests,
            data: HashMap::new(),
            broadcast_sock: None,
            port: 0
        };

        let bx: Arc<RwLock<Network>> = Arc::new(RwLock::new(net));

        Network::start_tcp_serv(bx.clone());
        Network::start_udp_serv(bx.clone());

        loop {

            println!("Waiting for servers to start....");

            let lcktmp: &RwLock<Network> = bx.borrow();
            let guard = lcktmp.read().unwrap();
            if let Some(_) = (*guard).broadcast_sock {
                if (*guard).port != 0 {
                    break;
                }
            }

            thread::sleep(time::Duration::from_millis(500));
        }

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

            println!("TCP Server running on port {}", port);

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

            println!("UDP Server running on port 52300");

            core.run(usrv).unwrap();
        });
    }


    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    fn broadcast_info(&self) {
        let addr = "255.255.255.255:52300".parse().unwrap();


        if let &Some(ref udpsock) = &self.broadcast_sock {
            match udpsock.send_to(protocol::get_broadcast(self.port, &self.interests).as_bytes(), &addr) {
                Ok(_) => return,
                Err(error) => println!("Error broadcasting {}", error)
            };
        } else {
            panic!("UDP not initialized! cannot broadcast before initialization is finished");
        }
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
                if let &Some(ref udpsock) = &(*guard).broadcast_sock {
                    input = try_nb!(udpsock.recv_from(&mut buf));
                } else {
                    panic!("UDP not initialized! This should not be reachable");
                }
            }


            print!("UDP received {:?} {:?}", buf, input);
        }
    }
}

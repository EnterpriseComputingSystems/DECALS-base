
extern crate futures;
extern crate tokio_io;
#[macro_use]
extern crate tokio_core;
extern crate net2;
extern crate rand;

use futures::{Future, Stream, Poll};

use tokio_core::net::{TcpListener, UdpSocket};
use tokio_core::reactor::Core;

use net2::UdpBuilder;

use std::collections::HashMap;
use std::{io, thread, time};
use std::sync::{RwLock, Arc};
use std::borrow::Borrow;
use std::net::{SocketAddr};

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//Local modules
mod protocol;
mod device;

use protocol::MsgData;

use device::Device;

const BROADCAST_PORT: u16 = 5320;

pub struct UDPServ {
    net: Arc<RwLock<Network>>
}

pub struct Network {
    num_devices: u32,
    data: HashMap<String, i32>,
    devices: HashMap<u64, Device>,
    interests: Vec<String>,
    broadcast_sock: Option<UdpSocket>,
    deviceid: u64,
    port: u16
}

impl Network {

    pub fn new(interests: Vec<String>)-> Arc<RwLock<Network>> {

        let net: Network = Network{num_devices: 0,
            interests: interests,
            data: HashMap::new(),
            devices: HashMap::new(),
            broadcast_sock: None,
            deviceid: rand::random::<u64>(),
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

            thread::sleep(time::Duration::from_millis(512));
        }

        thread::sleep(time::Duration::from_millis(1000));

        Network::start_heartbeat(bx.clone());

        println!("Servers Started");

        return bx;
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // TCP
    fn start_tcp_serv(network: Arc<RwLock<Network>>) {

        thread::Builder::new().name("tcp_serv".to_string()).spawn(|| {

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
        }).unwrap();

    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //UDP
    fn start_udp_serv(network: Arc<RwLock<Network>>) {

        thread::Builder::new().name("udp_serv".to_string()).spawn(|| {

            let net: Arc<RwLock<Network>> = network;
            let udpaddr = SocketAddr::from(([127, 0, 0, 1], BROADCAST_PORT));

            //Build socket
            let builder: UdpBuilder = UdpBuilder::new_v4().unwrap();

            #[cfg(unix)]
            builder.reuse_port(true).unwrap();

            let stdsock: std::net::UdpSocket = match builder.bind(&udpaddr) {
                Ok(sock) => sock,
                Err(error) => panic!("Couldn't listen for UDP! {}", error)
            };

            let mut core = Core::new().unwrap();
            let handle = core.handle();


            let broadcast_sock: UdpSocket = match UdpSocket::from_socket(stdsock, &handle) {
                Ok(sock) => sock,
                Err(error) => panic!("Couldn't convert UDP socket! {}", error)
            };

            broadcast_sock.set_broadcast(true).unwrap();

            {
                let lcktmp: &RwLock<Network> = net.borrow();
                let mut guard = lcktmp.write().unwrap();
                (*guard).broadcast_sock = Some(broadcast_sock);
            }

            let usrv = UDPServ{net: net};

            println!("UDP Server running on port {}", BROADCAST_PORT);

            core.run(usrv).unwrap();
        }).unwrap();
    }

    fn start_heartbeat(network: Arc<RwLock<Network>>) {

        thread::Builder::new().name("udp_serv".to_string()).spawn(|| {

            let net: Arc<RwLock<Network>> = network;

            loop {
                let lcktmp: &RwLock<Network> = net.borrow();
                let guard = lcktmp.read().unwrap();
                (*guard).broadcast_info();

                thread::sleep(time::Duration::from_millis(3000));
            }

        }).unwrap();
    }


    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    fn broadcast_info(&self) {
        let addr = format!("255.255.255.255:{}", BROADCAST_PORT).parse().unwrap();


        if let &Some(ref udpsock) = &self.broadcast_sock {
            println!("Broadcasting....");
            match udpsock.send_to(protocol::get_broadcast(self.deviceid, self.port, &self.interests).as_bytes(), &addr) {
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
        loop {
            let mut buf = vec![0; 1024];
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

            let msg: String;

            match String::from_utf8(buf) {
                Ok(s) => msg = s.trim().to_string(),
                Err(e) =>{
                    println!("UDP Broadcast: Received invalid UTF: {}", e);
                    continue;
                }
            };

            print!("UDP received from {:?} : {} ->", input, msg);

            if protocol::is_broadcast(&msg) {
                match protocol::parse_broadcast(&msg) {
                    MsgData::INVALID(er)=> {
                        println!("Error parsing hello - {}", er);
                    },
                    MsgData::HELLO(deviceid, port, interests)=> {

                    }
                }
            } else {
                println!("Unrecognized message type");
            }


        }
    }
}


extern crate futures;
extern crate tokio_io;
#[macro_use]
extern crate tokio_core;
extern crate net2;
extern crate rand;

// use futures::{Future, Stream, Poll};

// use tokio_core::net::{TcpListener, UdpSocket, TcpStream};
// use tokio_core::reactor::Core;

use net2::UdpBuilder;

#[cfg(unix)]
use net2::unix::UnixUdpBuilderExt;

use std::collections::HashMap;
use std::{io, thread, time};
use std::sync::{RwLock, Arc};
use std::borrow::Borrow;
use std::net::{SocketAddr, UdpSocket, TcpListener, TcpStream};

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//Local modules
mod protocol;
mod device;

use protocol::MsgData;

use device::Device;

const BROADCAST_PORT: u16 = 5320;

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

        Network::start_heartbeat(bx.clone());

        println!("Servers Started");

        return bx;
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // TCP
    fn start_tcp_serv(network: Arc<RwLock<Network>>) {


        let net: Arc<RwLock<Network>> = network;

        let tcpaddr: SocketAddr = "127.0.0.1:0".parse().unwrap();

        let tcp_listener = match TcpListener::bind(&tcpaddr) {
            Ok(lst) => lst,
            Err(error) => panic!("Couldn't listen for TCP! {}", error)
        };


        let port = tcp_listener.local_addr().unwrap().port();

        {
            let lcktmp: &RwLock<Network> = net.borrow();
            let mut guard = lcktmp.write().unwrap();
            (*guard).port = port;
        }

        thread::Builder::new().name("tcp_serv".to_string()).spawn(move || {

            loop {
                match tcp_listener.accept() {
                    Ok((sock, addr))=>{
                        thread::spawn(move || {handle_tcp_connection(&net.clone(), sock, addr)});
                    },
                    Err(e)=>println!("Connection from unknown host failed")
                }
            }
        }).unwrap();

        println!("TCP Server running on port {}", port);
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //UDP
    fn start_udp_serv(network: Arc<RwLock<Network>>) {
        let udpaddr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], BROADCAST_PORT));

        //Build socket
        let builder: UdpBuilder = UdpBuilder::new_v4().unwrap();

        #[cfg(unix)]
        builder.reuse_port(true).unwrap();

        let broadcast_sock: UdpSocket = match builder.bind(&udpaddr) {
            Ok(sock) => sock,
            Err(error) => panic!("Couldn't listen for UDP! {}", error)
        };

        broadcast_sock.set_broadcast(true).unwrap();

        {
            let lcktmp: &RwLock<Network> = network.borrow();
            let mut guard = lcktmp.write().unwrap();
            (*guard).broadcast_sock = Some(broadcast_sock);
        }

        thread::Builder::new().name("udp_serv".to_string()).spawn(|| {

            let net: Arc<RwLock<Network>> = network;
            loop {
                let mut buf = vec![0; 1024];
                let input;
                {
                    let lcktmp: &RwLock<Network> = net.borrow();
                    let guard = lcktmp.read().unwrap();
                    if let &Some(ref udpsock) = &(*guard).broadcast_sock {
                        match udpsock.recv_from(&mut buf) {
                            Ok(inp)=>input = inp,
                            Err(e)=>{
                                println!("Error receiving UDP: {}", e);
                                continue;
                            }
                        }
                    } else {
                        panic!("UDP not initialized! This should not be reachable");
                    }
                }

                match String::from_utf8(buf) {
                    Ok(s) => handle_udp_message(&net, s.trim().to_string(), input.1),
                    Err(e) =>{
                        println!("UDP Broadcast: Received invalid UTF: {}", e);
                        continue;
                    }
                };

            }
        }).unwrap();

        println!("UDP Server running on port {}", BROADCAST_PORT);
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
        let addr: SocketAddr = SocketAddr::from(([255, 255, 255, 255], BROADCAST_PORT));


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

fn handle_udp_message(net: &Arc<RwLock<Network>>, msg: String, addr: SocketAddr) {

    print!("UDP received from {:?} : {} -> ", addr, msg);

    if protocol::is_broadcast(&msg) {
        match protocol::parse_broadcast(&msg) {
            MsgData::INVALID(er)=> {
                println!("Error parsing hello - {}", er);
            },
            MsgData::HELLO(deviceid, port, interests)=> {

                let exists;
                {
                    let lcktmp: &RwLock<Network> = net.borrow();
                    let guard = lcktmp.read().unwrap();
                    exists = (*guard).devices.contains_key(&deviceid);
                }

                if exists {
                    println!("device already known");
                } else {

                    let newdev = Device::new(deviceid, SocketAddr::new(addr.ip(), port), interests);

                    {
                        let lcktmp: &RwLock<Network> = net.borrow();
                        let mut guard = lcktmp.write().unwrap();
                        (*guard).devices.insert(deviceid, newdev);
                    }

                    println!("device with id {} added", deviceid);


                }
            }
        }
    } else {
        println!("Unrecognized message type");
    }
}


fn handle_tcp_connection(network: &Arc<RwLock<Network>>, sock: TcpStream, addr: SocketAddr)->Result<(), String> {


    return Ok(());
}

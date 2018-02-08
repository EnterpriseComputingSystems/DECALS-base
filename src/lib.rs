
extern crate net2;
extern crate rand;

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
const HEARTBEAT_DELAY: u64 = 3000;

pub struct Network {
    num_devices: u32,
    data: RwLock<HashMap<String, String>>,
    devices: RwLock<HashMap<u64, Device>>,
    interests: Vec<String>,
    broadcast_sock: UdpSocket,
    deviceid: u64,
    port: u16
}

impl Network {

    pub fn new(interests: Vec<String>)-> Arc<Network> {


        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // TCP initialization
        let tcpaddr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 0));

        let tcp_listener = match TcpListener::bind(&tcpaddr) {
            Ok(lst) => lst,
            Err(error) => panic!("Couldn't listen for TCP! {}", error)
        };


        let port = tcp_listener.local_addr().unwrap().port();
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // UDP initialization
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
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~



        let new_net: Network = Network{num_devices: 0,
            interests: interests,
            data: RwLock::new(HashMap::new()),
            devices: RwLock::new(HashMap::new()),
            broadcast_sock: broadcast_sock,
            deviceid: rand::random::<u64>(),
            port: port
        };

        let net: Arc<Network> = Arc::new(new_net);

        Network::start_tcp_serv(tcp_listener, net.clone());
        println!("TCP Server running on port {}", port);

        Network::start_udp_serv(net.clone());
        println!("UDP Server running on port {}", BROADCAST_PORT);

        Network::start_heartbeat(net.clone());

        println!("Servers Started");

        return net;
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // TCP
    fn start_tcp_serv(tcp_listener: TcpListener, network: Arc<Network>) {



        thread::Builder::new().name("tcp_serv".to_string()).spawn(move || {

            let net: Arc<Network> = network;
            loop {
                match tcp_listener.accept() {
                    Ok((sock, addr))=>{
                        let netclone = net.clone();
                        thread::spawn(move || {
                            let net = netclone;
                            handle_tcp_connection(&net, sock, addr)});
                    },
                    Err(e)=>println!("Connection from unknown host failed")
                }
            }
        }).unwrap();
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //UDP
    fn start_udp_serv(network: Arc<Network>) {

        thread::Builder::new().name("udp_serv".to_string()).spawn(|| {

            let net: Arc<Network> = network;
            loop {
                let mut buf = vec![0; 1024];
                let input;
                {
                    let nettmp: &Network = net.borrow();
                    match nettmp.broadcast_sock.recv_from(&mut buf) {
                        Ok(inp)=>input = inp,
                        Err(e)=>{
                            println!("Error receiving UDP: {}", e);
                            continue;
                        }
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
    }

    fn start_heartbeat(network: Arc<Network>) {

        thread::Builder::new().name("udp_serv".to_string()).spawn(|| {

            let net: Arc<Network> = network;

            loop {
                net.broadcast_info();
                thread::sleep(time::Duration::from_millis(HEARTBEAT_DELAY));
            }

        }).unwrap();
    }


    pub fn get_num_devices(&self) ->u32 {
        self.num_devices
    }

    fn broadcast_info(&self) {
        let addr: SocketAddr = SocketAddr::from(([255, 255, 255, 255], BROADCAST_PORT));

        println!("Broadcasting....");
        match self.broadcast_sock.send_to(protocol::get_broadcast(self.deviceid, self.port, &self.interests).as_bytes(), &addr) {
            Ok(_) => return,
            Err(error) => println!("Error broadcasting {}", error)
        };
    }


}

fn handle_udp_message(net: &Arc<Network>, msg: String, addr: SocketAddr) {

    print!("UDP received from {:?} : {} -> ", addr, msg);

    if protocol::is_broadcast(&msg) {
        match protocol::parse_broadcast(&msg) {
            MsgData::INVALID(er)=> {
                println!("Error parsing hello - {}", er);
            },
            MsgData::HELLO(deviceid, port, interests)=> {

                let exists;
                {
                    let guard = net.devices.read().unwrap();
                    exists = (*guard).contains_key(&deviceid);
                }

                if exists {
                    println!("device already known");
                } else {

                    let newdev = Device::new(deviceid, SocketAddr::new(addr.ip(), port), interests);

                    {
                        let mut guard = net.devices.write().unwrap();
                        (*guard).insert(deviceid, newdev);
                    }

                    println!("device with id {} added", deviceid);
                }
            }
        }
    } else {
        println!("Unrecognized message type");
    }
}


fn handle_tcp_connection(network: &Arc<Network>, sock: TcpStream, addr: SocketAddr) {

}

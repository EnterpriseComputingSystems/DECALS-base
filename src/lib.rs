
extern crate net2;
extern crate rand;
extern crate time;
#[macro_use]
extern crate log;


#[cfg(unix)]
use net2::unix::UnixUdpBuilderExt;
use net2::UdpBuilder;

use std::collections::HashMap;
use std::{thread};
use std::time as stdtime;
use std::sync::{RwLock, Arc, Mutex};
use std::net::{SocketAddr, UdpSocket, TcpListener, TcpStream};
use std::io::BufReader;
use std::io::BufRead;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//Local modules
pub mod support;


mod protocol;
mod device;
mod data;
mod event;

use event::Event;

use protocol::MsgData;

use device::Device;

use data::DataPoint;


const BROADCAST_PORT: u16 = 5320;
const HEARTBEAT_DELAY: u64 = 3000;

pub struct Network {
    data: RwLock<HashMap<String, DataPoint>>,
    devices: RwLock<HashMap<u64, Device>>,
    interests: Vec<String>,
    broadcast_sock: UdpSocket,
    deviceid: u64,
    port: u16,
    event_sender: Mutex<Sender<Event>>,
    pub event_receiver: Mutex<Receiver<Event>>
}

impl Network {

    pub fn new(interests: Vec<String>)-> Arc<Network> {


        info!("Starting DECALS...");


        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // TCP initialization
        let tcpaddr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 0));

        let tcp_listener = match TcpListener::bind(&tcpaddr) {
            Ok(lst) => lst,
            Err(error) => panic!("Couldn't listen for TCP! {}", error)
        };


        let port = tcp_listener.local_addr().unwrap().port();
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // UDP initialization
        let udpaddr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], BROADCAST_PORT));

        //Build socket
        let builder: UdpBuilder = UdpBuilder::new_v4().unwrap();


        builder.reuse_address(true).unwrap();

        #[cfg(unix)]
        builder.reuse_port(true).unwrap();

        let broadcast_sock: UdpSocket = match builder.bind(&udpaddr) {
            Ok(sock) => sock,
            Err(error) => panic!("Couldn't listen for UDP! {}", error)
        };

        broadcast_sock.set_broadcast(true).unwrap();
        //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        let (send, rec) = mpsc::channel();


        let new_net: Network = Network{
            interests: interests,
            data: RwLock::new(HashMap::new()),
            devices: RwLock::new(HashMap::new()),
            broadcast_sock: broadcast_sock,
            deviceid: rand::random::<u64>(),
            port: port,
            event_sender: Mutex::new(send),
            event_receiver: Mutex::new(rec)
        };

        let net: Arc<Network> = Arc::new(new_net);

        info!("Starting TCP server...");
        Network::start_tcp_serv(net.clone(), tcp_listener);
        info!("TCP Server running on port {}", port);

        info!("Starting UDP server...");
        Network::start_udp_serv(net.clone());
        info!("UDP Server running on port {}", BROADCAST_PORT);

        info!("Starting heartbeat...");
        Network::start_heartbeat(net.clone());

        info!("Servers Started");

        return net;
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // TCP
    fn start_tcp_serv(net: Arc<Network>, tcp_listener: TcpListener) {

        thread::Builder::new().name("tcp_serv".to_string()).spawn(move || {

            loop {
                match tcp_listener.accept() {
                    Ok((sock, addr))=>{
                        let netclone = net.clone();
                        thread::spawn(move || {
                            Network::handle_tcp_connection(&netclone, sock, addr)});
                    },
                    Err(e)=>println!("Connection from unknown host failed {}", e)
                }
            }
        }).expect("Error starting tcp listener thread");
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //UDP
    fn start_udp_serv(net: Arc<Network>) {

        thread::Builder::new().name("udp_serv".to_string()).spawn(move || {

            loop {
                let mut buf = vec![0; 1024];
                match net.broadcast_sock.recv_from(&mut buf) {
                    Ok((size, addr))=>Network::handle_udp_message(&net, buf, size, addr),
                    Err(e)=>println!("Error receiving UDP: {}", e)
                }
            }
        }).expect("Error starting udp listener thread");
    }

    // Start a process to send out via udp broadcast this servers information
    fn start_heartbeat(network: Arc<Network>) {

        thread::Builder::new().name("decals_heartbeat".to_string()).spawn(|| {

            let net: Arc<Network> = network;

            loop {
                net.broadcast_info();
                thread::sleep(stdtime::Duration::from_millis(HEARTBEAT_DELAY));
            }

        }).expect("Error starting heartbeat thread");
    }

    //Broadcast over udp this device's information
    fn broadcast_info(&self) {
        let addr: SocketAddr = SocketAddr::from(([255, 255, 255, 255], BROADCAST_PORT));

        info!("Broadcasting....");
        match self.broadcast_sock.send_to(protocol::get_broadcast(self.deviceid, self.port, &self.interests).as_bytes(), &addr) {
            Ok(_) => return,
            Err(error) => println!("Error broadcasting {}", error)
        };
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //Data management

    // Get the number of discovered devices
    pub fn get_num_devices(&self) ->usize {
        let guard = self.devices.read().unwrap();
        return (*guard).len();
    }

    //Conveinience function to get the value of a key from the data map
    pub fn get_data_value(&self, key: &String)->String {
        let guard = self.data.read().unwrap();
        match (*guard).get(key.as_str()) {
            Some(s)=>return s.get_value(),
            None=>return String::new()
        }
    }

    //Set the value of a data point and update relevant external devices
    pub fn change_data_value(network: &Arc<Network>, key: String, val: String) {

        let net = network.clone();

        thread::spawn(move || {

            let tme = time::get_time();
            let datpt = DataPoint::new(key, val, tme);

            info!("Sending data update: {:?}", datpt);

            {
                let mut guard = net.data.write().unwrap();
                data::update_data_point(&mut (*guard), datpt.clone());
            }

            {
                let guard = net.devices.read().unwrap();
                for (_, device) in (*guard).iter() {
                    match device.send_data(datpt.clone()) {
                        Err(e)=>error!("Error sending to device {:?}: {}", device, e),
                        _=>{}
                    }
                }
            }
        });
    }


    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //UDP handling
    fn handle_udp_message(net: &Arc<Network>, buf: Vec<u8>, size: usize, addr: SocketAddr) {

        let msg = match String::from_utf8(buf) {
            Ok(s) => s[..size].trim().to_string(),
            Err(e) =>{
                warn!("UDP Broadcast: Received invalid UTF: {}", e);
                return;
            }
        };

        info!("UDP received from {:?} : {} -> ", addr, msg);

        if protocol::is_broadcast(&msg) {
            match protocol::parse_broadcast(&msg) {
                MsgData::INVALID(er)=> {
                    error!("Error parsing hello - {}", er);
                },
                MsgData::HELLO(deviceid, port, interests)=> {

                    if deviceid == net.deviceid {
                        info!("this device");
                    } else {

                        let exists;
                        {
                            let guard = net.devices.read().unwrap();
                            exists = (*guard).contains_key(&deviceid);
                        }

                        if exists {
                            info!("device already known");
                        } else {

                            let newdev = Device::new(deviceid, SocketAddr::new(addr.ip(), port), interests);

                            {
                                let mut guard = net.devices.write().unwrap();
                                (*guard).insert(deviceid, newdev);
                            }

                            {
                                let sender = net.event_sender.lock().unwrap();
                                sender.send(Event::UnitDiscovered(deviceid)).unwrap();
                            }

                            info!("device with id {} added", deviceid);
                        }
                    }
                },
                _=>warn!("Message not HELLO")
            }
        } else {
            warn!("Unrecognized message type");
        }
    }

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //TCP handling
    fn handle_tcp_connection(network: &Arc<Network>, sock: TcpStream, addr: SocketAddr) {

        let mut reader = BufReader::new(sock);


        info!("TCP connection from {:?} -> ", addr);

        loop {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Ok(size)=> {
                    if size == 0 {
                        break;
                    }

                    match protocol::parse_message(&buf) {
                        MsgData::DATA_SET(dp)=>{

                            info!("Updated data {:?}", dp);

                            {
                                let mut guard = network.data.write().unwrap();
                                data::update_data_point(&mut (*guard), dp.clone());
                            }

                            {
                                let sender = network.event_sender.lock().unwrap();
                                sender.send(Event::DataChange(dp)).unwrap();
                            }



                        },
                        MsgData::INVALID(e)=>error!("Error parsing incoming TCP message: {}", e),
                        _=>warn!("Unsupported incoming TCP message")
                    }
                },
                Err(e)=>{
                    error!("Error with incoming TCP connection: {}", e);
                    break;
                }
            }
        }

    }

}

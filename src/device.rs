

use std::net::{SocketAddr};

use data::DataPoint;


pub struct Device {
    deviceid: u64,
    addr: SocketAddr,
    interests: Vec<String>
}

impl Device {
    pub fn new(deviceid: u64, addr: SocketAddr, interests: Vec<String>)-> Device {
        let newdev = Device{deviceid: deviceid, addr: addr, interests: interests};

        return newdev;
    }

    pub fn send_data(&self, dat: DataPoint) {

    }
}

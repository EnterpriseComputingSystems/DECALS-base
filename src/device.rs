

use std::net::{SocketAddr};



pub struct Device {
    id: u64,
    addr: SocketAddr,
    interests: Vec<String>
}

impl Device {
    pub fn new(id: u64, addr: SocketAddr, interests: Vec<String>)-> Device {
        let newdev = Device{id: id, addr: addr, interests: interests};

        return newdev;
    }
}

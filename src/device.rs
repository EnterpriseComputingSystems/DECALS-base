
use std::io::Write;
use std::io;
use std::net::{SocketAddr, TcpStream};

use data::DataPoint;
use protocol;

#[derive(Debug)]
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

    pub fn send_data(&self, data: Vec<DataPoint>)->io::Result<()> {
        match TcpStream::connect(self.addr.clone()) {
            Ok(mut st)=> {
                for dat in data {
                    if let Err(e) = st.write_all(protocol::get_set_data(dat).as_bytes()) {
                        return Err(e);
                    }
                }
                Ok(())
            },
            Err(e)=>Err(e)
        }
    }
}

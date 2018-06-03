
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

    pub fn send(&self, data: &[u8])->io::Result<()> {
        match TcpStream::connect(self.addr.clone()) {
            Ok(mut st)=> {
                if let Err(e) = st.write_all(data) {
                    return Err(e);
                }
                Ok(())
            },
            Err(e)=>Err(e)
        }
    }

    pub fn send_string(&self, data: String)->io::Result<()> {
        self.send(data.as_bytes())
    }

    pub fn send_data(&self, data: Vec<DataPoint>)->io::Result<()> {
        self.send_string(data.into_iter().map(|dp| {protocol::get_set_data(dp)}).collect())
    }
}



mod protocol {


    pub fn get_hello(ip: &str, port: u16, mac: &str, interests: Vec<String>)->String {

        let mut output: String = format!("HELLO {}:{} {} [", ip, port, mac);

        let first = true;
        for inter in interests.iter().clone() {

            if !first { output = output + ","; }

            output = output + inter;
        }

        return output;
    }

    pub fn get_broadcast(ip: &str, port: u16, mac: &str, interests: Vec<String>)->String {
        return get_hello(ip, port, mac, interests);
    }


}

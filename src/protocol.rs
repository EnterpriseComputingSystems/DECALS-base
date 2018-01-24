


pub fn get_hello(port: u16, interests: &Vec<String>)->String {

    let mut output: String = format!("HELLO {} [", port);

    let first = true;
    for inter in interests.iter().clone() {

        if !first { output = output + ","; }

        output = output + inter;
    }

    return output + "]";
}

pub fn get_broadcast(port: u16, interests: &Vec<String>)->String {
    return get_hello(port, interests);
}

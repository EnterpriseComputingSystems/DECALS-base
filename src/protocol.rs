
pub enum MsgData {
    HELLO(u16, Vec<String>),
    INVALID(String)
}

pub fn get_hello(port: u16, interests: &Vec<String>)->String {

    let mut output: String = format!("HELLO {} [", port);

    let first = true;
    for inter in interests.iter().clone() {

        if !first { output = output + ","; }

        output = output + inter;
    }

    return output + "]";
}

pub fn is_hello(msg: &String)->bool {
    return msg.starts_with("HELLO ");
}

pub fn parse_hello(message: &String)->MsgData {

    let mut msg = message.clone();

    assert!(is_hello(&msg));

    //Remove header
    msg = msg.split_off(7);

    //Get port
    let div = msg.find(' ').unwrap();
    let mut intrstr = msg.split_off(div);

    //Remove " ["
    intrstr = msg.split_off(2);

    //Remove "]"
    intrstr.pop();

    let port = match msg.parse() {
        Ok(num)=> num,
        Err(error)=> return MsgData::INVALID(format!("Thought HELLO, err={}", error))
    };

    return MsgData::HELLO(port, intrstr.split(',').map(|s| s.to_string()).collect());
}

pub fn get_broadcast(port: u16, interests: &Vec<String>)->String {
    return get_hello(port, interests);
}

pub fn is_broadcast(msg: &String)->bool {
    return is_hello(msg);
}

pub fn parse_broadcast(msg: &String)->MsgData {
    return parse_hello(msg);
}

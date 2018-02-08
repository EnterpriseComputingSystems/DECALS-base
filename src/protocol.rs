
pub enum MsgData {
    HELLO(u64, u16, Vec<String>),
    INVALID(String)
}

//Get data from an unknown incoming message.
pub fn parse_message(message: &String)->MsgData {

    if is_broadcast(&message) {
        return parse_broadcast(&message);
    }

    if is_hello(&message) {
        return parse_hello(&message);
    }

    return MsgData::INVALID(format!("Unknown message type: {}", message));
}

pub fn get_hello(deviceid: u64, port: u16, interests: &Vec<String>)->String {

    let mut output: String = format!("HELLO {} {} [", deviceid, port);

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
    msg = msg.split_off(6);

    let mut parts = msg.split_whitespace();

    let port: u16;
    let deviceid: u64;

    //Get device id
    match parts.next() {
        Some(pt)=> deviceid = match pt.parse() {
                    Ok(num)=> num,
                    Err(error)=> return MsgData::INVALID(format!("Error parsing HELLO: {}", error))
                },
        None=>return MsgData::INVALID("Error parsing HELLO: Missing deviceid".to_string())
    };

    //Get port
    match parts.next() {
        Some(pt)=> port = match pt.parse() {
                    Ok(num)=> num,
                    Err(error)=> return MsgData::INVALID(format!("Error parsing HELLO: {}", error))
                },
        None=>return MsgData::INVALID("Error parsing HELLO: Missing port".to_string())
    };

    let mut intrstr;

    //Get interests
    match parts.next() {
        Some(pt)=> {
            intrstr = pt.to_string().split_off(1); // Remove leading and tailing brackets
            intrstr.pop();
        },
        None=>return MsgData::INVALID("Error parsing HELLO: Missing port".to_string())
    };

    return MsgData::HELLO(deviceid, port, intrstr.split(',').map(|s| s.to_string()).collect());
}

pub fn get_broadcast(deviceid: u64, port: u16, interests: &Vec<String>)->String {
    return get_hello(deviceid, port, interests);
}

pub fn is_broadcast(msg: &String)->bool {
    return is_hello(msg);
}

pub fn parse_broadcast(msg: &String)->MsgData {
    return parse_hello(msg);
}



//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// TESTS

#[cfg(test)]
mod protocol_tests {

    use super::*;

    #[test]
    fn test_broadcast_prot() {
        let mut interests: Vec<String> = Vec::new();
        interests.push("Test".to_string());

        match parse_broadcast(&get_broadcast(1234567890, 1234, &interests)) {
            MsgData::HELLO(did, p, i)=> {
                assert_eq!(did, 1234567890);
                assert_eq!(p, 1234);
                assert_eq!(interests, i);},
            MsgData::INVALID(err)=> panic!(err)
        }
    }

    #[test]
    fn test_hello_prot() {
        let mut interests: Vec<String> = Vec::new();
        interests.push("Test".to_string());

        match parse_hello(&get_hello(1234567890, 1234, &interests)) {
            MsgData::HELLO(did, p, i)=> {
                assert_eq!(did, 1234567890);
                assert_eq!(p, 1234);
                assert_eq!(interests, i);},
            MsgData::INVALID(err) => panic!(MsgData::INVALID(err))
        }
    }
}

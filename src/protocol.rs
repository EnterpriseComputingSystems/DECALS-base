use time::Timespec;
use time;

use data::DataPoint;

pub enum MsgData {
    HELLO(u64, u16, Vec<String>),
    DATA_SET(DataPoint),
    REGISTER_SETTING(String, Vec<String>),
    INVALID(String)
}


//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//HELLO
pub fn get_hello(deviceid: u64, port: u16, interests: &Vec<String>)->String {

    let mut output: String = format!("HELLO {} {} [", deviceid, port);

    let mut first = true;
    for inter in interests.iter().clone() {

        if !first { output = output + ","; }

        output = output + inter;

        first = false;
    }

    return output + "]\n";
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

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//BROADCAST
pub fn get_broadcast(deviceid: u64, port: u16, interests: &Vec<String>)->String {
    return get_hello(deviceid, port, interests);
}

pub fn is_broadcast(msg: &String)->bool {
    return is_hello(msg);
}

pub fn parse_broadcast(msg: &String)->MsgData {
    return parse_hello(msg);
}




//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//DATA
pub fn get_set_data(dat: DataPoint)->String {
    format!("DATA SET {} {} {}\n", dat.key, dat.value, encode_timespec(&dat.timestamp))
}

pub fn is_data(msg: &String)->bool {
    return msg.starts_with("DATA ");
}

pub fn parse_data(message: &String)->MsgData {

    let mut msg = message.clone();

    assert!(is_data(&msg));

    //Remove header
    msg = msg.split_off(5);

    let mut parts = msg.split_whitespace();

    let key: String;
    let value: String;
    let time: Timespec;

    let set: bool;

    //Get action
    match parts.next() {
        Some(pt)=> {
            if pt == "READ" {
                set = false;
            } else if pt == "SET" {
                set = true;
            } else {
                return MsgData::INVALID("Error parsing DATA: Unknown action".to_string());
            }
        },
        None=>return MsgData::INVALID("Error parsing DATA: Missing action".to_string())
    };

    //Get key
    match parts.next() {
        Some(pt)=> key = pt.to_string(),
        None=>return MsgData::INVALID("Error parsing DATA: Missing key".to_string())
    };

    //Get value
    match parts.next() {
        Some(pt)=> value = pt.to_string(),
        None=>return MsgData::INVALID("Error parsing DATA: Missing value".to_string())
    };

    //Get time
    match parts.next() {
        Some(pt)=> match decode_timespec(pt.to_string()) {
            Ok(t)=>time=t,
            Err(e)=>return MsgData::INVALID(format!("Error parsing DATA: time parse error -> {}", e))
        },
        None=>return MsgData::INVALID("Error parsing DATA: Missing value".to_string())
    };

    if set {
        return MsgData::DATA_SET(DataPoint::new(key, value, time));
    } else {
        return MsgData::INVALID("Other data actions not supported yet".to_string());
    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Register setting
const REGISTER_SETTING_HEADER: &str = "REGISTER SETTING ";

pub fn get_register_setting(key: String, options: &Vec<String>)->String {

    let mut output: String = format!("{}{} [", REGISTER_SETTING_HEADER, key);

    let mut first = true;
    for inter in options.iter().clone() {

        if !first { output = output + ","; }

        output = output + inter;

        first = false;
    }

    return output + "]\n";
}

pub fn is_register_setting(msg: &String)->bool {
    return msg.starts_with(REGISTER_SETTING_HEADER);
}

pub fn parse_register_setting(mut msg: String)->MsgData {

    assert!(is_register_setting(&msg));

    //Remove header
    let mut data = msg.split_off(REGISTER_SETTING_HEADER.len());

    let mut options: String = match data.find(' ') {
        Some(idx)=>data.split_off(idx).trim().to_string(),
        None=>return MsgData::INVALID("Error parsing REGISTER_SETTING: Missing key".to_string())
    };

    options = options.split_off(1); //Remove brackets
    options.pop();

    return MsgData::REGISTER_SETTING(data, options.split(',').map(|s| s.to_string()).collect());
}


//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Helpers

//Get data from an unknown incoming message.
pub fn parse_message(message: &String)->MsgData {

    if is_broadcast(&message) {
        return parse_broadcast(&message);
    }

    if is_hello(&message) {
        return parse_hello(&message);
    }

    if is_data(&message) {
        return parse_data(&message);
    }

    if is_register_setting(&message) {
        return parse_register_setting(message.clone());
    }

    return MsgData::INVALID(format!("Unknown message type: {}", message));
}

fn encode_timespec(ts: &Timespec)->String {
    format!("{}:{}", ts.sec, ts.nsec)
}

fn decode_timespec(dat: String)->Result<Timespec, String> {

    let mut split = dat.split(':');

    let sec: i64;
    let nano: i32;

    match split.next() {
        Some(s)=> match s.parse() {
            Ok(n)=>sec=n,
            Err(e)=>return Err(format!("couldn't parse seconds: {}", e))
        },
        None=>return Err("missing seconds".to_string())
    }

    match split.next() {
        Some(s)=> match s.parse() {
            Ok(n)=>nano=n,
            Err(e)=>return Err(format!("couldn't parse nanoseconds: {}", e))
        },
        None=>return Err("missing nanoseconds".to_string())
    }

    return Ok(Timespec::new(sec, nano));
}


//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Tests

#[cfg(test)]
mod protocol_tests {

    use super::*;

    #[test]
    fn test_timespec_encode() {
        let tme = time::get_time();
        assert_eq!(tme, decode_timespec(encode_timespec(&tme)).unwrap());
    }

    #[test]
    fn test_broadcast_prot() {
        let mut interests: Vec<String> = Vec::new();
        interests.push("Test".to_string());
        interests.push("Test2".to_string());

        match parse_broadcast(&get_broadcast(1234567890, 1234, &interests)) {
            MsgData::HELLO(did, p, i)=> {
                assert_eq!(did, 1234567890);
                assert_eq!(p, 1234);
                assert_eq!(interests, i);},
            MsgData::INVALID(err)=> panic!(err),
            _=>panic!("Wrong MsgData")
        }
    }

    #[test]
    fn test_hello_prot() {
        let mut interests: Vec<String> = Vec::new();
        interests.push("Test".to_string());
        interests.push("Test2".to_string());

        match parse_hello(&get_hello(1234567890, 1234, &interests)) {
            MsgData::HELLO(did, p, i)=> {
                assert_eq!(did, 1234567890);
                assert_eq!(p, 1234);
                assert_eq!(interests, i);},
            MsgData::INVALID(err) => panic!(err),
            _=>panic!("Wrong MsgData")
        }
    }

    #[test]
    fn test_data_set_prot() {

        let tme = time::get_time();
        let dp = DataPoint::new("key".to_string(), "value".to_string(), tme);
        match parse_data(&get_set_data(dp.clone())) {
            MsgData::DATA_SET(dp2)=> assert_eq!(dp, dp2),
            MsgData::INVALID(err) => panic!(err),
            _=>panic!("Wrong MsgData")
        }
    }

    #[test]
    fn test_register_setting_prot() {

        let mut options: Vec<String> = Vec::new();
        options.push("Test".to_string());
        options.push("Test 2".to_string());

        match parse_register_setting(get_register_setting("settingkey".to_string(), &options)) {
            MsgData::REGISTER_SETTING(key, opt)=> {
                assert_eq!(key, "settingkey".to_string());
                assert_eq!(options, opt);},
            MsgData::INVALID(err) => panic!(err),
            _=>panic!("Wrong MsgData")
        }
    }
}

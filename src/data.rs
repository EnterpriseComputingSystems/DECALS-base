
use time::Timespec;

use std::collections::HashMap;
use std::cmp::PartialEq;


#[derive(Debug, Clone)]
pub struct DataPoint {
    pub key: String,
    pub value: String,
    pub timestamp: Timespec
}

impl DataPoint {

    pub fn new(key: String, value: String, timestamp: Timespec)->DataPoint {
        DataPoint{key, value, timestamp}
    }

    pub fn clone(&self)->DataPoint {
        DataPoint::new(self.key.clone(), self.value.clone(), self.timestamp)
    }

    pub fn get_value(&self)->String {
        return self.value.clone();
    }

    pub fn check_set_value(&mut self, dp: DataPoint) {
        if self.is_before(&dp) {
            self.timestamp = dp.timestamp;
            self.value = dp.value;
        }
    }

    pub fn is_before(&self, other: &DataPoint)->bool {
        return self.timestamp < other.timestamp;
    }


}

impl PartialEq for DataPoint {
    fn eq(&self, other: &DataPoint)->bool {
        return self.key == other.key &&
            self.value == other.value &&
            self.timestamp == other.timestamp;
    }
}


pub fn update_data_point(dat: &mut HashMap<String, DataPoint>, datpt: DataPoint) {
    match dat.get_mut(datpt.key.as_str()) {
        Some(dp)=>{
            dp.check_set_value(datpt);
            return;
        },
        None=>{}
    }
    dat.insert(datpt.key.clone(), datpt);
}

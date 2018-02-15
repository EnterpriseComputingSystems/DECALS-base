
use time::Timespec;

use std::collections::HashMap;

pub struct DataPoint {
    key: String,
    value: String,
    timestamp: Timespec
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


use time::Timespec;

use std::collections::HashMap;

pub struct DataPoint {
    timestamp: Timespec,
    value: String
}

impl DataPoint {

    pub fn get_value(&self)->String {
        return self.value.clone();
    }

    pub fn check_set_value(&mut self, val: String, ts: Timespec) {
        if self.timestamp < ts {
            self.timestamp = ts;
            self.value = val;
        }
    }


}


pub fn update_data_point(dat: &mut HashMap<String, DataPoint>, key: String, val: String, ts: Timespec) {
    let ent = dat.entry(key).or_insert(DataPoint{timestamp: ts, value: val.clone()});
    (*ent).check_set_value(val, ts);
}

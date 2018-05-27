
use time::Timespec;

use std::collections::{HashMap};
use std::cmp::PartialEq;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use time;



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




#[derive(Debug, Clone)]
pub struct DataReference {
    pub current_value: Arc<RwLock<DataPoint>>,
    last_time: Timespec,
    data_manager: DataManager
}

impl DataReference {
    pub fn new(value: Arc<RwLock<DataPoint>>, data_manager: DataManager)->DataReference {
        DataReference{current_value: value, data_manager, last_time: time::Timespec::new(0, 0)}
    }

    pub fn from_dp(dp: DataPoint, data_manager: DataManager)->DataReference {
        DataReference::new(Arc::new(RwLock::new(dp)), data_manager)
    }

    pub fn update_data(&self, val: String) {
        let mut dp = self.current_value.write().unwrap();
        dp.value = val;
        dp.timestamp = time::get_time();
        self.data_manager.set_entry_dirty(dp.key.clone());
    }

    pub fn get_data(&self)->DataPoint {
        self.current_value.read().unwrap().clone()
    }

    pub fn get_key(&self)->String {
        self.current_value.read().unwrap().key.clone()
    }

    pub fn get_value(&self)->String {
        self.current_value.read().unwrap().value.clone()
    }

    pub fn test_changed(&mut self)->bool {
        let ts = self.current_value.read().unwrap().timestamp;
        if self.last_time != ts {
            self.last_time = ts;
            return true;
        }

        return false;
    }
}

#[derive(Debug, Clone)]
pub struct DataManager {
    data: Arc<RwLock<HashMap<String, DataReference>>>,
    dirty_list_sender: Arc<Mutex<Sender<String>>>,
    dirty_list_receiver: Arc<Mutex<Receiver<String>>>
}


impl DataManager {

    pub fn new()->DataManager {
        let (send, recv) = mpsc::channel();
        DataManager{data: Arc::new(RwLock::new(HashMap::new())),
            dirty_list_sender: Arc::new(Mutex::new(send)),
            dirty_list_receiver: Arc::new(Mutex::new(recv))}
    }

    fn create_new_entry(&self, key: &str)->DataReference {
        DataReference::from_dp(DataPoint::new(key.to_string(), String::new(), time::get_time()), self.clone())
    }

    pub fn get_reference(&self, key:  &String)->DataReference {
        let mut data = self.data.write().unwrap();
        data.entry(key.clone()).or_insert(self.create_new_entry(&key)).clone()
    }

    pub fn get_datapoint(&self, key: &String)->DataPoint {
        self.data.write().unwrap().entry(key.clone()).or_insert(self.create_new_entry(&key)).get_data()
    }

    /// Updates a data point just locally, so without marking it dirty to be broadcast
    pub fn update_data_point_locally(&self, datpt: DataPoint) {
        let mut data = self.data.write().unwrap();

        if let Some(dp) = data.get_mut(datpt.key.as_str()) {
            dp.current_value.write().unwrap().check_set_value(datpt);
            return;
        }

        data.insert(datpt.key.clone(), DataReference::from_dp(datpt, self.clone()));
    }

    /// Updates a data point and marks it dirty
    pub fn update_data_point(&self, datpt: DataPoint) {

        self.update_data_point_locally(datpt.clone());
        self.set_entry_dirty(datpt.key);

    }

    pub fn get_all_data(&self)->Vec<DataPoint> {
        self.data.read().unwrap().iter().map(|(_, dr)| dr.get_data()).collect()
    }

    pub fn set_entry_dirty(&self, key: String) {
        self.dirty_list_sender.lock().unwrap().send(key).unwrap();
    }

    pub fn get_dirty_entry(&self)->DataPoint {
        let key = self.dirty_list_receiver.lock().unwrap().recv().unwrap();
        if let Some(val) = self.data.read().unwrap().get(&key) {
            return val.get_data();
        } else {
            panic!("dirty entry doesnt exist");
        }
    }

}

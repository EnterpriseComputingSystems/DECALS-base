
use data::DataPoint;

pub enum Event {
    DataChange(DataPoint),
    SettingRegistered(String, Vec<String>),
    UnitDiscovered(u64)
}

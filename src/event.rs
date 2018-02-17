
use data::DataPoint;

pub enum Event {
    DataChange(DataPoint),
    UnitDiscovered(u64)
}

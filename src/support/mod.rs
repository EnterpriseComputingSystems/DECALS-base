

pub mod alert {

    pub const ALERT_KEY: &str = "ALERT_STATUS";

    #[derive(Clone, Copy)]
    pub enum Alert{
        Normal,
        Yellow,
        Red,
        Blue,
        Black,

    }

    pub fn get_alert_text(al: Alert)->String {
        match al {
            Alert::Normal => "Normal".to_string(),
            Alert::Yellow => "Yellow".to_string(),
            Alert::Red => "Red".to_string(),
            Alert::Blue => "Blue".to_string(),
            Alert::Black => "Black".to_string(),

        }
    }
}

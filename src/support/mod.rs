

/// A helper module to provide a common basis for transmitting alert status information.
pub mod alert {

    /// A common key to be used for use DECALS to synchronize alert status.
    pub const ALERT_KEY: &str = "ALERT_STATUS";

    /// Represents the various possible 'Alert' states a system could be in, a la StarTrek.
    #[derive(Clone, Copy, PartialEq)]
    pub enum Alert{
        Normal,
        Yellow,
        Red,
        Blue,
        Black,

    }

    /// Get a human-readable text representation of an alert
    pub fn get_alert_text(al: Alert)->String {
        match al {
            Alert::Normal => "Normal".to_string(),
            Alert::Yellow => "Yellow".to_string(),
            Alert::Red => "Red".to_string(),
            Alert::Blue => "Blue".to_string(),
            Alert::Black => "Black".to_string(),

        }
    }

    /// Get an alert from a text string. Defaults to 'Normal'
    pub fn get_alert_from_text(al: String)->Alert {
        match al.as_str() {
            "Normal" => Alert::Normal,
            "Yellow" => Alert::Yellow,
            "Red" => Alert::Red,
            "Blue" => Alert::Blue,
            "Black" => Alert::Black,
            _ => Alert::Normal
        }
    }
}

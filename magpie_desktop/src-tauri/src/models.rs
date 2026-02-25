use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OmniTicket {
    pub vehicle_code: String,
    pub vehicle_type: String,
    pub booking_status: String,
    pub start_time: String,
    pub arrive_time: String,
    pub duration: String,
    pub price_info: String,
    pub from_station_name: String,
    pub to_station_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StationInfo {
    pub city: String,
    pub train_code: String,
    pub flight_code: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserConfig {
    pub persona: String,
    pub time_window_start: String,
    pub time_window_end: String,
    pub departure: StationInfo,
    pub destinations: Vec<StationInfo>,
    pub budget_cap: i32,
}

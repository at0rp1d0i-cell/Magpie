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
    
    // One-way or round-trip
    #[serde(default = "default_trip_type")]
    pub trip_type: String, // "one_way" | "round_trip"
    
    pub time_window_start: String,
    pub time_window_end: String,
    
    // Optional return window
    pub return_time_window_start: Option<String>,
    pub return_time_window_end: Option<String>,
    
    #[serde(default = "default_passenger_count")]
    pub passenger_count: i32,

    pub departure: StationInfo,
    pub destinations: Vec<StationInfo>,
    pub budget_cap: i32,
}

fn default_trip_type() -> String {
    "one_way".to_string()
}
fn default_passenger_count() -> i32 {
    1
}

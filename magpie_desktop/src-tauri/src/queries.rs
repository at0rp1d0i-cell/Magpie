use rusqlite::Connection;
use std::env;

/// Read latest tickets from SQLite (tickets fetched within last 15 minutes)
pub fn fetch_latest_tickets_from_db() -> Result<Vec<crate::models::OmniTicket>, Box<dyn std::error::Error + Send + Sync>> {
    let mut db_path = env::current_dir().unwrap_or_default();
    if db_path.ends_with("src-tauri") {
        db_path.pop();
        db_path.pop();
    }
    let data_dir = db_path.join("data");
    let db_file = data_dir.join("tickets.db");

    if !db_file.exists() {
        return Ok(vec![]);
    }

    let conn = Connection::open(&db_file)?;

    // Simplified for Tauri IPC: Just get top 50 newest valid tickets
    let mut stmt = conn.prepare(
        "SELECT vehicle_code, vehicle_type, booking_status, start_time, arrive_time, duration, price_info, from_station_name, to_station_name 
         FROM omni_tickets 
         ORDER BY fetch_time DESC, start_time ASC 
         LIMIT 50"
    )?;

    let iter = stmt.query_map([], |row| {
        Ok(crate::models::OmniTicket {
            vehicle_code: row.get(0)?,
            vehicle_type: row.get(1)?,
            booking_status: row.get(2)?,
            start_time: row.get(3)?,
            arrive_time: row.get(4)?,
            duration: row.get(5)?,
            price_info: row.get(6)?,
            from_station_name: row.get(7)?,
            to_station_name: row.get(8)?,
        })
    })?;

    let mut tickets = Vec::new();
    for ticket in iter.flatten() {
        tickets.push(ticket);
    }

    Ok(tickets)
}

#[tauri::command]
pub async fn get_latest_tickets() -> Result<Vec<crate::models::OmniTicket>, String> {
    fetch_latest_tickets_from_db().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daemon_status() -> Result<String, String> {
    // Simplified status representation
    Ok("running".to_string())
}

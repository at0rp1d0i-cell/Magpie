use rusqlite::Connection;
use std::env;

use tauri::{AppHandle, Manager};

/// Read latest tickets from SQLite (tickets fetched within last 15 minutes)
pub fn fetch_latest_tickets_from_db(app: &AppHandle) -> Result<Vec<crate::models::OmniTicket>, Box<dyn std::error::Error + Send + Sync>> {
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| env::current_dir().unwrap());
    let data_dir = app_dir.join("data");
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
pub async fn get_latest_tickets(app: tauri::AppHandle) -> Result<Vec<crate::models::OmniTicket>, String> {
    fetch_latest_tickets_from_db(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_daemon_status(app: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| env::current_dir().unwrap());
    let db_file = app_dir.join("data").join("tickets.db");

    if !db_file.exists() {
        return Ok("Idle (No DB)".to_string());
    }

    let conn = match Connection::open(&db_file) {
        Ok(c) => c,
        Err(_) => return Ok("Active (DB Locked)".to_string()),
    };

    let mut stmt = match conn.prepare("SELECT fetch_time FROM omni_tickets ORDER BY fetch_time DESC LIMIT 1") {
        Ok(s) => s,
        Err(_) => return Ok("Active (Query Error)".to_string()),
    };

    let mut rows = match stmt.query([]) {
        Ok(r) => r,
        Err(_) => return Ok("Active (Query Error)".to_string()),
    };

    if let Ok(Some(row)) = rows.next() {
        if let Ok(fetch_time) = row.get::<_, String>(0) {
            return Ok(format!("同步于 {}", fetch_time));
        }
    }

    Ok("Waiting for first cycle".to_string())
}

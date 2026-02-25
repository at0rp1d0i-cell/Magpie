use crate::models::OmniTicket;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub fn init_db(db_path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS omni_tickets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fetch_time TEXT NOT NULL,
            travel_date TEXT NOT NULL,
            from_station_name TEXT NOT NULL,
            to_station_name TEXT NOT NULL,
            vehicle_code TEXT NOT NULL,
            vehicle_type TEXT NOT NULL,
            booking_status TEXT NOT NULL,
            start_time TEXT NOT NULL,
            arrive_time TEXT NOT NULL,
            duration TEXT NOT NULL,
            price_info TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_ticket(
    conn: &Connection,
    ticket: &OmniTicket,
    fetch_time: &str,
    travel_date: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO omni_tickets (fetch_time, travel_date, from_station_name, to_station_name, vehicle_code, vehicle_type, booking_status, start_time, arrive_time, duration, price_info)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            fetch_time,
            travel_date,
            ticket.from_station_name,
            ticket.to_station_name,
            ticket.vehicle_code,
            ticket.vehicle_type,
            ticket.booking_status,
            ticket.start_time,
            ticket.arrive_time,
            ticket.duration,
            ticket.price_info
        ],
    )?;
    Ok(())
}

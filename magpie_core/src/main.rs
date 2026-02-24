use chrono::Local;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize, Serialize)]
struct TrainTicket {
    train_code: String,
    booking_status: String,
    start_time: String,
    arrive_time: String,
    duration: String,
    second_class: String,
    first_class: String,
    business_class: String,
    no_seat: String,
}

fn init_db(db_path: &PathBuf) -> SqliteResult<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS train_tickets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fetch_time TEXT NOT NULL,
            train_date TEXT NOT NULL,
            from_station TEXT NOT NULL,
            to_station TEXT NOT NULL,
            train_code TEXT NOT NULL,
            booking_status TEXT NOT NULL,
            start_time TEXT NOT NULL,
            arrive_time TEXT NOT NULL,
            duration TEXT NOT NULL,
            second_class TEXT NOT NULL,
            first_class TEXT NOT NULL,
            business_class TEXT NOT NULL,
            no_seat TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

async fn fetch_tickets(date: &str, from: &str, to: &str) -> Result<Vec<TrainTicket>, Box<dyn Error>> {
    let mut agent_dir = env::current_dir()?;
    agent_dir.pop();
    agent_dir.push("magpie_agent");

    let output = Command::new("uv")
        .arg("run")
        .arg("train_monitor.py")
        .arg("--date")
        .arg(date)
        .arg("--from_station")
        .arg(from)
        .arg("--to_station")
        .arg(to)
        .current_dir(&agent_dir)
        .output()
        .await?;

    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        for line in stderr_str.lines() {
            eprintln!("{}", line);
        }
    }

    if !output.status.success() {
        return Err(format!("Python script failed with status: {}", output.status).into());
    }

    let stdout_str = String::from_utf8(output.stdout)?;
    // output usually contains JSON lines, let's just parse it directly
    if stdout_str.trim().is_empty() {
         return Ok(vec![]);
    }
    
    let tickets: Vec<TrainTicket> = serde_json::from_str(&stdout_str)?;
    Ok(tickets)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🐦 Magpie Core - High-Frequency Dispatcher Started");

    let mut db_path = env::current_dir()?;
    db_path.pop(); // Go up to /Magpie
    let data_dir = db_path.join("data");
    std::fs::create_dir_all(&data_dir)?;
    let db_file = data_dir.join("tickets.db");

    let conn = init_db(&db_file)?;
    println!("📦 Database initialized at: {:?}", db_file);

    let date = "2026-03-01";
    let from = "BJP"; // Beijing
    let to = "NCG";   // Nanchang

    let fetch_interval = Duration::from_secs(60); // Demo: every 60s
    let mut cycle = 1;

    // We only loop a few times for this MVP testing phase, but in production this is loop {}
    loop {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("\n[{}] ⏳ Cycle {}: Calling Python Agent for {} -> {} on {}...", now, cycle, from, to, date);
        
        match fetch_tickets(date, from, to).await {
            Ok(tickets) => {
                let mut inserted = 0;
                for t in &tickets {
                    // Filter out unbookable trains to save DB space
                    if t.booking_status == "Y" {
                        conn.execute(
                            "INSERT INTO train_tickets (fetch_time, train_date, from_station, to_station, train_code, booking_status, start_time, arrive_time, duration, second_class, first_class, business_class, no_seat)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                            params![
                                now, date, from, to, 
                                t.train_code, t.booking_status, t.start_time, t.arrive_time, 
                                t.duration, t.second_class, t.first_class, t.business_class, t.no_seat
                            ],
                        )?;
                        inserted += 1;
                    }
                }
                println!("✅ Cycle {} Complete: Received {} trains, inserted {} valid records to SQLite.", cycle, tickets.len(), inserted);
            },
            Err(e) => {
                eprintln!("❌ Cycle {} Failed: {}", cycle, e);
            }
        }

        println!("💤 Sleeping for {} seconds...", fetch_interval.as_secs());
        sleep(fetch_interval).await;
        cycle += 1;
        
        // Break after 1 cycle for demo purposes, so the process doesn't hang in CI/sandbox
        // Remove this break for the real background daemon.
        break; 
    }

    Ok(())
}

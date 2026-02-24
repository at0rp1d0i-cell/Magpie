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

#[derive(Debug, Deserialize)]
struct UserConfig {
    persona: String,
    time_window_start: String,
    #[allow(dead_code)]
    time_window_end: String,
    destinations: Vec<String>,
    budget_cap: i32,
}

fn load_user_config(path: &PathBuf) -> Result<UserConfig, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: UserConfig = serde_json::from_str(&content)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🐦 Magpie Core - High-Frequency Dispatcher Started");

    let mut db_path = env::current_dir()?;
    db_path.pop(); // Go up to /Magpie
    let data_dir = db_path.join("data");
    std::fs::create_dir_all(&data_dir)?;
    let db_file = data_dir.join("tickets.db");
    let config_file = data_dir.join("user_config.json");

    let conn = init_db(&db_file)?;
    println!("📦 Database initialized at: {:?}", db_file);

    // We only loop a few times for this MVP testing phase, but in production this is loop {}
    #[allow(clippy::never_loop)]
    loop {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 动态热重载用户配置单 (Hot Reload Strategy)
        let config = match load_user_config(&config_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Warning] Failed to load config from {:?}: {}. Using default fallback.", config_file, e);
                // 默认提供一个安全的防御性配置
                UserConfig {
                    persona: "leisure".to_string(),
                    time_window_start: Local::now().format("%Y-%m-%d").to_string(),
                    time_window_end: Local::now().format("%Y-%m-%d").to_string(),
                    destinations: vec!["NCG".to_string()],
                    budget_cap: 9999,
                }
            }
        };

        // TODO: Map natural Chinese city names from DeepSeek to Railway Codes (e.g. 南京 -> NJH)
        // MVP directly uses the first destination name for station query.
        let from = "BJP"; // Beijing
        let to = &config.destinations[0];   // Naive mapping
        let date = &config.time_window_start; // Only watching start date in MVP V1

        // 意图引擎核心：根据 LLM 打的画像标签动态改变爬取频率
        let fetch_interval = if config.persona.to_lowercase() == "business" {
            println!("\n[Intent Strategy] 🧑‍💼 Persona: Business -> Active polling every 60s.");
            Duration::from_secs(60)
        } else {
            println!("\n[Intent Strategy] 🍹 Persona: Leisure -> Winter mode polling every 10800s (3 hours).");
            Duration::from_secs(10800)
        };

        println!("[{}] ⏳ Cycle: Calling Python Agent for {} -> {} on {} with Budget Cap ￥{}...", now, from, to, date, config.budget_cap);
        
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
                println!("✅ Cycle Complete: Received {} trains, inserted {} valid records to SQLite.", tickets.len(), inserted);
            },
            Err(e) => {
                eprintln!("❌ Cycle Failed: {}", e);
            }
        }

        println!("💤 Sleeping for {} seconds...\n", fetch_interval.as_secs());
        sleep(fetch_interval).await;
        
        // Break after 1 cycle for demo purposes, so the process doesn't hang in CI/sandbox
        // Remove this break for the real background daemon.
        break; 
    }

    Ok(())
}

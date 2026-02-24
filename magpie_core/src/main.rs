use chrono::Local;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize, Serialize)]
struct OmniTicket {
    vehicle_code: String,
    vehicle_type: String,
    booking_status: String,
    start_time: String,
    arrive_time: String,
    duration: String,
    price_info: String,
    from_station_name: String,
    to_station_name: String,
}

fn init_db(db_path: &PathBuf) -> SqliteResult<Connection> {
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

async fn fetch_tickets(date: &str, from_city: &str, to_city: &str, script_name: &str) -> Result<Vec<OmniTicket>, Box<dyn Error>> {
    let mut agent_dir = env::current_dir()?;
    agent_dir.pop();
    agent_dir.push("magpie_agent");

    // Rust maps the conceptual from/to to the CLI flags required by the scripts.
    let (from_flag, to_flag) = if script_name == "train_monitor.py" {
        ("--from_station", "--to_station")
    } else {
        ("--from_city", "--to_city")
    };

    let output = Command::new("uv")
        .arg("run")
        .arg(script_name)
        .arg("--date")
        .arg(date)
        .arg(from_flag)
        .arg(from_city)
        .arg(to_flag)
        .arg(to_city)
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
    
    let tickets: Vec<OmniTicket> = serde_json::from_str(&stdout_str)?;
    Ok(tickets)
}

#[derive(Debug, Deserialize)]
struct StationInfo {
    city: String,
    train_code: String,
    flight_code: String,
}

#[derive(Debug, Deserialize)]
struct UserConfig {
    persona: String,
    time_window_start: String,
    #[allow(dead_code)]
    time_window_end: String,
    departure: StationInfo,
    destinations: Vec<StationInfo>,
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
                    departure: StationInfo {
                        city: "北京".to_string(),
                        train_code: "BJP".to_string(),
                        flight_code: "BJS".to_string(),
                    },
                    destinations: vec![StationInfo {
                        city: "南昌".to_string(),
                        train_code: "NCG".to_string(),
                        flight_code: "KHN".to_string(),
                    }],
                    budget_cap: 9999,
                }
            }
        };

        // 动态加载 LLM 注入的机读代码
        let from_train = &config.departure.train_code;
        let to_train = &config.destinations[0].train_code;

        let from_flight = &config.departure.flight_code;
        let to_flight = &config.destinations[0].flight_code;

        let date = &config.time_window_start; // Only watching start date in MVP V1

        // 意图引擎核心：根据 LLM 打的画像标签动态改变爬取频率
        let fetch_interval = if config.persona.to_lowercase() == "business" {
            println!("\n[Intent Strategy] 🧑‍💼 Persona: Business -> Active polling every 60s.");
            Duration::from_secs(60)
        } else {
            println!("\n[Intent Strategy] 🍹 Persona: Leisure -> Winter mode polling every 10800s (3 hours).");
            Duration::from_secs(10800)
        };

        println!("[{}] ⏳ Cycle: Calling Python Agent for {} -> {} on {} with Budget Cap ￥{}...", now, from_train, to_train, date, config.budget_cap);
        
        match fetch_tickets(date, from_train, to_train, "train_monitor.py").await {
            Ok(tickets) => {
                let mut inserted = 0;
                for t in &tickets {
                    // Filter out unbookable trains to save DB space
                    if t.booking_status == "Y" {
                        conn.execute(
                            "INSERT INTO omni_tickets (fetch_time, travel_date, from_station_name, to_station_name, vehicle_code, vehicle_type, booking_status, start_time, arrive_time, duration, price_info)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                            params![
                                now, date, t.from_station_name, t.to_station_name, 
                                t.vehicle_code, t.vehicle_type, t.booking_status, t.start_time, t.arrive_time, 
                                t.duration, t.price_info
                            ],
                        )?;
                        inserted += 1;
                    }
                }
                println!("✅ Cycle Complete: Received {} trains, inserted {} valid records to SQLite.", tickets.len(), inserted);
            },
            Err(e) => {
                eprintln!("❌ Train Cycle Failed: {}", e);
            }
        }

        // Also run flight fetch
        println!("[{}] ⏳ Cycle: Calling Flight Agent for {} -> {} on {}...", now, from_flight, to_flight, date);
        match fetch_tickets(date, from_flight, to_flight, "flight_monitor.py").await {
            Ok(tickets) => {
                let mut inserted = 0;
                for t in &tickets {
                    if t.booking_status == "Y" {
                        conn.execute(
                            "INSERT INTO omni_tickets (fetch_time, travel_date, from_station_name, to_station_name, vehicle_code, vehicle_type, booking_status, start_time, arrive_time, duration, price_info)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                            params![
                                now, date, t.from_station_name, t.to_station_name, 
                                t.vehicle_code, t.vehicle_type, t.booking_status, t.start_time, t.arrive_time, 
                                t.duration, t.price_info
                            ],
                        )?;
                        inserted += 1;
                    }
                }
                println!("✅ Cycle Complete: Received {} flights, inserted {} valid records to SQLite.", tickets.len(), inserted);
            },
            Err(e) => {
                eprintln!("❌ Flight Cycle Failed: {}", e);
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

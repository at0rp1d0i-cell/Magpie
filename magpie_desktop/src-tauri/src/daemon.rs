use crate::db;
use crate::decision::run_decision_engine;
use crate::fetchers::{flight::query_variflight, train::query_12306};
use crate::models::{StationInfo, UserConfig};
use chrono::{Local, NaiveDate, Duration as ChronoDuration};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

fn load_user_config(path: &PathBuf) -> Result<UserConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: UserConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub async fn start_background_task(trigger: Arc<Notify>) {
    println!("🐦 Magpie Desktop - High-Frequency Dispatcher Started");

    let mut db_path = env::current_dir().unwrap_or_default();
    // Navigate upwards just like the original logic, or strictly use current_dir relative to binary.
    // In Tauri dev mode, current_dir is usually `src-tauri`, but the data dir was mapped to Workspace/Magpie/data.
    // Let's ensure data dir resolves safely.
    if db_path.ends_with("src-tauri") {
        db_path.pop(); // map to magpie_desktop
        db_path.pop(); // map to Magpie
    } // else we are packaged app, data is next to binary usually (or should use tauri appDataDir ideally)

    let data_dir = db_path.join("data");
    let _ = std::fs::create_dir_all(&data_dir);
    let db_file = data_dir.join("tickets.db");
    let config_file = data_dir.join("user_config.json");

    let conn = db::init_db(&db_file).expect("Failed to initialize database");
    println!("📦 Database initialized at: {:?}", db_file);

    let env_path = db_path.join(".env");
    let _ = dotenv::from_path(env_path);

    loop {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let config = match load_user_config(&config_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[Warning] Failed to load config from {:?}: {}. Using default fallback.",
                    config_file, e
                );
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

        if config.destinations.is_empty() {
            println!("⚠️ No destinations configured (destinations array is empty). Skipping fetch cycle.");
            tokio::select! {
                _ = sleep(fetch_interval) => {},
                _ = trigger.notified() => {
                    println!("⚡ Instant Fetch TriggerReceived! Waking up immediately...");
                }
            }
            continue;
        }

        let from_train = &config.departure.train_code;
        let to_train = &config.destinations[0].train_code;

        let from_flight = &config.departure.flight_code;
        let to_flight = &config.destinations[0].flight_code;

        let fetch_interval = if config.persona.to_lowercase() == "business" {
            println!("\n[Intent Strategy] 🧑‍💼 Persona: Business -> Active polling every 60s.");
            Duration::from_secs(60)
        } else {
            println!(
                "\n[Intent Strategy] 🍹 Persona: Leisure -> Winter mode polling every 10800s (3 hours)."
            );
            Duration::from_secs(10800)
        };

        let mut omni_tickets = Vec::new();
        
        let start_date = NaiveDate::parse_from_str(&config.time_window_start, "%Y-%m-%d").unwrap_or_else(|_| Local::now().naive_local().date());
        let end_date = NaiveDate::parse_from_str(&config.time_window_end, "%Y-%m-%d").unwrap_or(start_date);
        
        let mut target_dates = Vec::new();
        let mut current_date = start_date;
        while current_date <= end_date && target_dates.len() < 30 {
            target_dates.push(current_date);
            current_date += ChronoDuration::days(1);
        }

        let today = Local::now().naive_local().date();

        for date_obj in target_dates {
            let days_ahead = (date_obj - today).num_days();
            if days_ahead < 0 {
                continue; // Skip past dates
            }
            let date_str = date_obj.format("%Y-%m-%d").to_string();

            // Train fetching
            if days_ahead < 15 {
                println!(
                    "[{}] ⏳ Cycle: Calling Native Rust Agent for Train {} -> {} on {} with Budget Cap ￥{}...",
                    now, from_train, to_train, date_str, config.budget_cap
                );
                match query_12306(&date_str, from_train, to_train).await {
                    Ok(tickets) => {
                        let mut inserted = 0;
                        for t in &tickets {
                            if t.booking_status == "Y" {
                                omni_tickets.push(t.clone());
                                if db::insert_ticket(&conn, t, &now, &date_str).is_ok() {
                                    inserted += 1;
                                }
                            }
                        }
                        println!(
                            "✅ Cycle Complete: Received {} trains, inserted {} valid records to SQLite.",
                            tickets.len(), inserted
                        );
                    }
                    Err(e) => eprintln!("❌ Train Cycle Failed: {}", e),
                }
            } else {
                println!("[{}] 🚆 Skipping train query for {} (beyond 15-day 12306 presale threshold).", now, date_str);
            }

            // Flight fetching
            println!(
                "[{}] ⏳ Cycle: Calling Native Rust Agent for Flight {} -> {} on {}...",
                now, from_flight, to_flight, date_str
            );

            match query_variflight(&date_str, from_flight, to_flight).await {
                Ok(tickets) => {
                    let mut inserted = 0;
                    for t in &tickets {
                        if t.booking_status == "Y" {
                            omni_tickets.push(t.clone());
                            if db::insert_ticket(&conn, t, &now, &date_str).is_ok() {
                                inserted += 1;
                            }
                        }
                    }
                    println!(
                        "✅ Cycle Complete: Received {} flights, inserted {} valid records to SQLite.",
                        tickets.len(), inserted
                    );
                }
                Err(e) => eprintln!("❌ Flight Cycle Failed: {}", e),
            }
        }

        if !omni_tickets.is_empty() {
            if let Err(e) = run_decision_engine(&omni_tickets, config.budget_cap).await {
                eprintln!("❌ Decision Engine Failed: {}", e);
            }
        } else {
            println!("⚠️ No valid tickets found across target dates to run Decision Engine.");
        }

        println!("💤 Sleeping for {} seconds...\n", fetch_interval.as_secs());
        tokio::select! {
            _ = sleep(fetch_interval) => {},
            _ = trigger.notified() => {
                println!("⚡ Instant Fetch TriggerReceived! Waking up immediately...");
            }
        }
    }
}

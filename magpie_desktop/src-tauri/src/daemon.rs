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
use tauri::{AppHandle, Manager};

fn load_user_config(path: &PathBuf) -> Result<UserConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: UserConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub async fn start_background_task(trigger: Arc<Notify>, app: AppHandle) {
    println!("🐦 Magpie Desktop - High-Frequency Dispatcher Started");

    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| env::current_dir().unwrap());

    let data_dir = app_dir.join("data");
    let _ = std::fs::create_dir_all(&data_dir);
    let db_file = data_dir.join("tickets.db");
    let config_file = data_dir.join("user_config.json");

    let conn = db::init_db(&db_file).expect("Failed to initialize database");
    println!("📦 Database initialized at: {:?}", db_file);

    let env_path = app_dir.join(".env");
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
                    trip_type: "one_way".to_string(),
                    time_window_start: Local::now().format("%Y-%m-%d").to_string(),
                    time_window_end: Local::now().format("%Y-%m-%d").to_string(),
                    return_time_window_start: None,
                    return_time_window_end: None,
                    passenger_count: 1,
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

        let fetch_interval = if config.persona.to_lowercase() == "business" {
            println!("\n[Intent Strategy] 🧑‍💼 Persona: Business -> Active polling every 60s.");
            Duration::from_secs(60)
        } else {
            println!(
                "\n[Intent Strategy] 🍹 Persona: Leisure -> Winter mode polling every 10800s (3 hours)."
            );
            Duration::from_secs(10800)
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

        struct TripLeg {
            from_train: String,
            to_train: String,
            from_flight: String,
            to_flight: String,
            start_date: NaiveDate,
            end_date: NaiveDate,
        }

        let mut legs = Vec::new();
        let start_date = NaiveDate::parse_from_str(&config.time_window_start, "%Y-%m-%d").unwrap_or_else(|_| Local::now().naive_local().date());
        let end_date = NaiveDate::parse_from_str(&config.time_window_end, "%Y-%m-%d").unwrap_or(start_date);
        
        legs.push(TripLeg {
            from_train: config.departure.train_code.clone(),
            to_train: config.destinations[0].train_code.clone(),
            from_flight: config.departure.flight_code.clone(),
            to_flight: config.destinations[0].flight_code.clone(),
            start_date,
            end_date,
        });

        if config.trip_type == "round_trip" {
            if let (Some(ret_start), Some(ret_end)) = (&config.return_time_window_start, &config.return_time_window_end) {
                let r_start = NaiveDate::parse_from_str(ret_start, "%Y-%m-%d").unwrap_or(end_date);
                let r_end = NaiveDate::parse_from_str(ret_end, "%Y-%m-%d").unwrap_or(r_start);
                legs.push(TripLeg {
                    from_train: config.destinations[0].train_code.clone(),
                    to_train: config.departure.train_code.clone(),
                    from_flight: config.destinations[0].flight_code.clone(),
                    to_flight: config.departure.flight_code.clone(),
                    start_date: r_start,
                    end_date: r_end,
                });
            }
        }

        let mut omni_tickets = Vec::new();
        let today = Local::now().naive_local().date();

        for leg in legs {
            let mut target_dates = Vec::new();
            let mut current_date = leg.start_date;
            while current_date <= leg.end_date && target_dates.len() < 30 {
                target_dates.push(current_date);
                current_date += ChronoDuration::days(1);
            }

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
                        now, leg.from_train, leg.to_train, date_str, config.budget_cap
                    );
                    match query_12306(&date_str, &leg.from_train, &leg.to_train).await {
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
                    now, leg.from_flight, leg.to_flight, date_str
                );

                match query_variflight(&date_str, &leg.from_flight, &leg.to_flight).await {
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

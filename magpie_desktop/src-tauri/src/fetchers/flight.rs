use crate::models::OmniTicket;
use regex::Regex;
use reqwest::{header, Client};
use serde_json::{json, Value};
use std::env;

pub async fn query_variflight(
    date: &str,
    from: &str,
    to: &str,
) -> Result<Vec<OmniTicket>, Box<dyn std::error::Error>> {
    let api_key = env::var("VARIFLIGHT_API_KEY")
        .map_err(|_| "VARIFLIGHT_API_KEY is missing in environment variables")?;

    let client = Client::builder().user_agent("Magpie/1.0").build()?;

    let body = json!({
        "endpoint": "searchFlightItineraries",
        "params": {
            "depCityCode": from,
            "arrCityCode": to,
            "depDate": date
        }
    });

    let res = client
        .post("https://mcp.variflight.com/api/v1/mcp/data")
        .header("X-VARIFLIGHT-KEY", api_key)
        .header(header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    let text = res.text().await?;
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);

    let mut tickets = Vec::new();

    if let Some(data_str) = json.get("data").and_then(|v| v.as_str()) {
        let re = Regex::new(
            r"航班号[：:]\s*([A-Z0-9]+).*?起飞时间[：:]\s*([\d\-: ]+).*?到达时间[：:]\s*([\d\-: ]+).*?耗时[：:]\s*([^，,\s]+).*?价格[：:]\s*(\d+)元",
        )?;

        for cap in re.captures_iter(data_str) {
            let start_time_full = cap.get(2).map_or("", |m| m.as_str());
            let arrive_time_full = cap.get(3).map_or("", |m| m.as_str());

            let start_time = if start_time_full.len() >= 16 {
                start_time_full[11..16].to_string()
            } else {
                start_time_full.to_string()
            };

            let arrive_time = if arrive_time_full.len() >= 16 {
                arrive_time_full[11..16].to_string()
            } else {
                arrive_time_full.to_string()
            };

            let ticket = OmniTicket {
                vehicle_code: cap.get(1).map_or("", |m| m.as_str()).to_string(),
                vehicle_type: "flight".to_string(),
                booking_status: "Y".to_string(),
                start_time,
                arrive_time,
                duration: cap.get(4).map_or("", |m| m.as_str()).to_string(),
                price_info: format!("￥{}", cap.get(5).map_or("0", |m| m.as_str())),
                from_station_name: from.to_string(),
                to_station_name: to.to_string(),
            };

            tickets.push(ticket);
        }

        if tickets.is_empty() && !data_str.trim().is_empty() {
            eprintln!("[Warning] VariFlight returned data but regex matched 0 tickets. Data snippet: {:.150}", data_str);
        }
    }

    Ok(tickets)
}

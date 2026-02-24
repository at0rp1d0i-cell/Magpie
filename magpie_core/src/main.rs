use dotenv::dotenv;
use reqwest::{Client, IntoUrl};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Deserialize, Debug)]
struct FlightOffersResponse {
    data: Vec<serde_json::Value>,
}

async fn get_token(client: &Client, client_id: &str, client_secret: &str) -> Result<String, Box<dyn Error>> {
    let url = "https://test.api.amadeus.com/v1/security/oauth2/token";
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let res: TokenResponse = client
        .post(url)
        .form(&params)
        .send()
        .await?
        .json()
        .await?;

    Ok(res.access_token)
}

async fn search_flights(client: &Client, token: &str, origin: &str, dest: &str, date: &str) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://test.api.amadeus.com/v2/shopping/flight-offers?originLocationCode={}&destinationLocationCode={}&departureDate={}&adults=1&nonStop=false&max=20",
        origin, dest, date
    );

    let res: FlightOffersResponse = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    println!("✅ 成功获取到 {}->{} ({}) 的航班报价，共有 {} 条结果", origin, dest, date, res.data.len());
    
    println!("{:<10} | {:<12} | {:<15} | {:<10} | {:<10}", "航司及航班", "起飞时间", "总价(EUR)", "预估(CNY@7.55)", "舱位级别");
    println!("-------------------------------------------------------------------------");

    for offer in res.data.iter() {
        let itins = offer["itineraries"].as_array().unwrap();
        let segments = itins[0]["segments"].as_array().unwrap();
        let first_seg = &segments[0];
        
        let carrier = first_seg["carrierCode"].as_str().unwrap_or("??");
        let number = first_seg["number"].as_str().unwrap_or("??");
        let departure = first_seg["departure"]["at"].as_str().unwrap_or("??");
        let departs_time = departure.split('T').last().unwrap_or(departure);
        
        let price_str = offer["price"]["total"].as_str().unwrap_or("0");
        let price: f64 = price_str.parse().unwrap_or(0.0);
        let cny = price * 7.55;
        
        let mut cabin_info = String::new();
        if let Some(pricings) = offer["travelerPricings"].as_array() {
            if let Some(fares) = pricings[0]["fareDetailsBySegment"].as_array() {
                let cabin = fares[0]["cabin"].as_str().unwrap_or("?");
                let class = fares[0]["class"].as_str().unwrap_or("?");
                cabin_info = format!("{}({})", cabin, class);
            }
        }

        println!("{:<13} | {:<12} | €{:<14.2} | ¥{:<13.2} | {}", 
            format!("{}{}", carrier, number), 
            departs_time, 
            price,
            cny,
            cabin_info
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let client_id = env::var("AMADEUS_CLIENT_ID")
        .expect("⚠️ 环境变量中未找到 AMADEUS_CLIENT_ID");
    let client_secret = env::var("AMADEUS_CLIENT_SECRET")
        .expect("⚠️ 环境变量中未找到 AMADEUS_CLIENT_SECRET");

    let client = Client::new();

    println!("⏳ 正在向 Amadeus 请求 OAuth_Token...");
    let token = get_token(&client, &client_id, &client_secret).await?;
    println!("🔑 获取 Token 成功！");

    // Test a flight next week (e.g., currently Feb 2026, let's search for Mar 01)
    let origin = "BJS"; // Beijing (Any airport)
    let dest = "KHN"; // Nanchang
    // TODO: dynamically get date
    let date = "2026-03-01"; 

    println!("⏳ 正在查询航班报价 (测试参数: {} 到 {}, 日期: {})...", origin, dest, date);
    search_flights(&client, &token, origin, dest, date).await?;

    Ok(())
}

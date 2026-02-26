use crate::models::OmniTicket;
use reqwest::{header, Client};
use serde_json::Value;
use std::sync::Arc;
use tokio::task::JoinSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn query_12306(
    date: &str,
    from: &str,
    to: &str,
) -> Result<Vec<OmniTicket>, Box<dyn std::error::Error>> {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .danger_accept_invalid_certs(true)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        .build()?;

    // 1. 初始化会话
    let _ = client
        .get("https://kyfw.12306.cn/otn/leftTicket/init")
        .send()
        .await?;

    // 2. 获取设备指纹 (logdevice) 
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string();
    let logdevice_url = "https://kyfw.12306.cn/otn/HttpZF/logdevice";
    let params = [
        ("algID", "WYEdoc45yu"),
        ("hashCode", "EhTtj7Znzyie6I21jpgekYReLAnA8fyGEB4VlIGbF0g"),
        ("FMQw", "0"),
        ("q4f3", "zh-CN"),
        ("VPIf", "1"),
        ("custID", "133"),
        ("VEek", "unknown"),
        ("hAqN", "Win32"),
        ("platform", "WEB"),
        ("TeRS", "728x1366"),
        ("tOHY", "24xx768x1366"),
        ("timestamp", &current_timestamp),
    ];

    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "text/javascript, application/javascript, application/ecmascript, application/x-ecmascript, */*; q=0.01".parse()?);
    headers.insert(
        header::REFERER,
        "https://kyfw.12306.cn/otn/passport?redirect=/otn/login/conf".parse()?,
    );
    headers.insert(header::HOST, "kyfw.12306.cn".parse()?);

    if let Ok(res) = client
        .get(logdevice_url)
        .headers(headers)
        .query(&params)
        .send()
        .await
    {
        if let Ok(text) = res.text().await {
            let mut dfp = String::new();
            let mut exp = String::new();

            if let Some(start) = text.find("\"dfp\":\"") {
                if let Some(end) = text[start + 7..].find('\"') {
                    dfp = text[start + 7..start + 7 + end].to_string();
                }
            } else if let Some(start) = text.find("'dfp':'") {
                if let Some(end) = text[start + 7..].find('\'') {
                    dfp = text[start + 7..start + 7 + end].to_string();
                }
            }

            if let Some(start) = text.find("\"exp\":\"") {
                if let Some(end) = text[start + 7..].find('\"') {
                    exp = text[start + 7..start + 7 + end].to_string();
                }
            } else if let Some(start) = text.find("'exp':'") {
                if let Some(end) = text[start + 7..].find('\'') {
                    exp = text[start + 7..start + 7 + end].to_string();
                }
            }

            let domain_url = "https://kyfw.12306.cn".parse::<reqwest::Url>()?;
            if !dfp.is_empty() {
                jar.add_cookie_str(
                    &format!("RAIL_DEVICEID={}; Domain=.12306.cn; Path=/", dfp),
                    &domain_url,
                );
            }
            if !exp.is_empty() {
                jar.add_cookie_str(
                    &format!("RAIL_EXPIRATION={}; Domain=.12306.cn; Path=/", exp),
                    &domain_url,
                );
            }
        }
    }

    // 3. 正式发出查票请求 (queryZ)
    let query_url = "https://kyfw.12306.cn/otn/leftTicket/queryZ";
    let mut query_headers = header::HeaderMap::new();
    query_headers.insert(header::HOST, "kyfw.12306.cn".parse()?);
    query_headers.insert(
        header::REFERER,
        "https://kyfw.12306.cn/otn/leftTicket/init".parse()?,
    );
    query_headers.insert("X-Requested-With", "XMLHttpRequest".parse()?);
    query_headers.insert("Sec-Fetch-Site", "same-origin".parse()?);

    let query_params = vec![
        ("leftTicketDTO.train_date", date),
        ("leftTicketDTO.from_station", from),
        ("leftTicketDTO.to_station", to),
        ("purpose_codes", "ADULT"),
    ];

    let query_res = client
        .get(query_url)
        .headers(query_headers.clone())
        .query(&query_params)
        .send()
        .await?;

    let query_text = query_res.text().await?;
    let json: Value = serde_json::from_str(&query_text).unwrap_or(Value::Null);

    let mut temp_tickets = Vec::new();
    
    // Parsed intermediate struct to hold data for fetching price
    struct TrainPriceMeta {
        vehicle_code: String,
        booking_status: String,
        start_time: String,
        arrive_time: String,
        duration: String,
        seat_2: String,
        seat_1: String,
        stand: String,
        train_no: String,
        from_station_no: String,
        to_station_no: String,
        seat_types: String,
        train_date: String,
    }

    if let Some(result_arr) = json
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.as_array())
    {
        for item in result_arr {
            if let Some(s) = item.as_str() {
                let parts: Vec<&str> = s.split('|').collect();
                if parts.len() >= 36 {
                    let mut dur = parts[10].to_string();
                    if let Some((h, m)) = dur.split_once(':') {
                        if let (Ok(hi), Ok(mi)) = (h.parse::<u32>(), m.parse::<u32>()) {
                           dur = format!("{}h{:02}m", hi, mi);
                        }
                    }
                    let seat_types = if !parts[35].is_empty() { parts[35] } else { parts[34] };
                    let meta = TrainPriceMeta {
                        vehicle_code: parts[3].to_string(),
                        booking_status: parts[11].to_string(),
                        start_time: parts[8].to_string(),
                        arrive_time: parts[9].to_string(),
                        duration: dur,
                        seat_2: parts[30].to_string(),
                        seat_1: parts[31].to_string(),
                        stand: parts[26].to_string(),
                        train_no: parts[2].to_string(),
                        from_station_no: parts[16].to_string(),
                        to_station_no: parts[17].to_string(),
                        seat_types: seat_types.to_string(),
                        train_date: date.to_string(),
                    };
                    temp_tickets.push(meta);
                }
            }
        }
    }

    // 4. Batch async fetch prices
    let mut join_set = JoinSet::new();
    
    // To protect against rate-limiting on 12306, we fetch the price concurrently
    for (idx, meta) in temp_tickets.into_iter().enumerate() {
        let client_clone = client.clone();
        let q_h = query_headers.clone();
        let from_clone = from.to_string();
        let to_clone = to.to_string();
        
        join_set.spawn(async move {
            let p_url = "https://kyfw.12306.cn/otn/leftTicket/queryTicketPrice";
            let p_params = [
                ("train_no", meta.train_no.as_str()),
                ("from_station_no", meta.from_station_no.as_str()),
                ("to_station_no", meta.to_station_no.as_str()),
                ("seat_types", meta.seat_types.as_str()),
                ("train_date", meta.train_date.as_str()),
            ];
            
            // Allow up to 3s per inner price request
            let price_req = client_clone
                .get(p_url)
                .headers(q_h)
                .query(&p_params)
                .timeout(std::time::Duration::from_secs(4))
                .send()
                .await;
                
            let mut extracted_price = String::new();
            if let Ok(r) = price_req {
                if let Ok(txt) = r.text().await {
                    let p_json: Value = serde_json::from_str(&txt).unwrap_or(Value::Null);
                    if let Some(data) = p_json.get("data") {
                        // Extract O (Second class), M (First class), A9 (Business) or WZ (Standing)
                        let raw_p = data.get("O").or(data.get("M")).or(data.get("WZ")).or(data.get("A9"));
                        if let Some(rp) = raw_p {
                            if let Some(s) = rp.as_str() {
                                extracted_price = s.to_string(); // e.g. "¥599.0"
                            }
                        }
                    }
                }
            }
            
            let final_price_str = if extracted_price.is_empty() {
                format!("暂无价格¦二等座:{}¦一等座:{}", meta.seat_2, meta.seat_1)
            } else {
                // Ensure the extracted price aligns with our parser
                let clean_price = extracted_price.replace("¥", "￥"); 
                format!("{} ¦ 二等座:{} ¦ 一等座:{}", clean_price, meta.seat_2, meta.seat_1)
            };

            let ticket = OmniTicket {
                vehicle_code: meta.vehicle_code,
                vehicle_type: "train".to_string(),
                booking_status: meta.booking_status,
                start_time: meta.start_time,
                arrive_time: meta.arrive_time,
                duration: meta.duration,
                price_info: final_price_str,
                from_station_name: from_clone,
                to_station_name: to_clone,
            };
            
            (idx, ticket)
        });
    }

    let mut final_tickets = vec![None; join_set.len()];
    while let Some(res) = join_set.join_next().await {
        if let Ok((idx, t)) = res {
            final_tickets[idx] = Some(t);
        }
    }

    let tickets: Vec<OmniTicket> = final_tickets.into_iter().flatten().collect();

    Ok(tickets)
}

use crate::models::OmniTicket;
use reqwest::{header, Client};
use serde_json::Value;
use std::sync::Arc;
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

    // 2. 获取设备指纹 (logdevice) 重写 Python 黑魔法
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

            // 简单而暴力的 JSON 提词器（等效 Python 的 eval）
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
        .headers(query_headers)
        .query(&query_params)
        .send()
        .await?;

    let query_text = query_res.text().await?;
    let json: Value = serde_json::from_str(&query_text).unwrap_or(Value::Null);

    let mut tickets = Vec::new();
    if let Some(result_arr) = json
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.as_array())
    {
        for item in result_arr {
            if let Some(s) = item.as_str() {
                let parts: Vec<&str> = s.split('|').collect();
                if parts.len() >= 32 {
                    let ticket = OmniTicket {
                        vehicle_code: parts[3].to_string(),
                        vehicle_type: "train".to_string(),
                        booking_status: parts[11].to_string(),
                        start_time: parts[8].to_string(),
                        arrive_time: parts[9].to_string(),
                        duration: parts[10].to_string(),
                        // 12306 queryZ does not return prices directly.
                        // [30]=second class seats, [31]=first class seats, [26]=standing seats
                        // These are availability counts ("有"/"无"/number), not prices.
                        price_info: format!(
                            "二等座:{}¦一等座:{}¦无座:{}",
                            parts[30], parts[31], parts[26]
                        ),
                        from_station_name: from.to_string(),
                        to_station_name: to.to_string(),
                    };
                    tickets.push(ticket);
                }
            }
        }
    }

    Ok(tickets)
}

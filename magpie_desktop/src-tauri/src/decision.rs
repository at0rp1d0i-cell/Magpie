use crate::models::OmniTicket;
use regex::Regex;
use reqwest::{header, Client};
use serde_json::{json, Value};
use std::env;

pub async fn run_decision_engine(
    tickets: &[OmniTicket],
    budget_cap: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    if tickets.is_empty() {
        println!("[Info] No active tickets found for decision engine.");
        return Ok(());
    }

    let mut valid_tickets = Vec::new();
    let re = Regex::new(r"￥(\d+(?:\.\d+)?)")?;

    for t in tickets {
        let price_str = &t.price_info;

        let mut has_valid_price = false;
        let pre_matches: Vec<_> = re.captures_iter(price_str).collect();
        if pre_matches.is_empty() {
            valid_tickets.push(t.clone());
        } else {
            for cap in pre_matches {
                if let Some(m) = cap.get(1) {
                    if let Ok(price) = m.as_str().parse::<f64>() {
                        if price <= budget_cap as f64 {
                            has_valid_price = true;
                            break;
                        }
                    }
                }
            }
            if has_valid_price {
                valid_tickets.push(t.clone());
            }
        }
    }

    if valid_tickets.is_empty() {
        println!("🔕 拦截机制触发: 监控到 {} 条空铁数据，但没有符合心理预算 (≤￥{}) 的可行方案，拦截推送。", tickets.len(), budget_cap);
        return Ok(());
    }

    println!(
        "✅ 从 {} 条数据中筛出 {} 条低于 ￥{} 预算的三维时空数据送往 LLM 决策...",
        tickets.len(),
        valid_tickets.len(),
        budget_cap
    );

    let system_prompt = "
    你是一个名叫 Magpie (鹊桥 Agent) 的高级差旅管家。你的职责不仅仅是比价，更重要的是提供极其专业且富含情绪价值的出行决策建议。
    目前你的用户是一位异地恋的高净值极客，他计划跨城过周末。你收到了下面最新的包含【高铁、飞机】的双轨余票监控快照。

    【任务要求】
    1. 你需要进行“空铁联合决策”，像一个真人秘书一样汇报当前的余票紧缺度或者低价情况。
    2. 如果高铁和飞机同在一个时间段，对比它们的时间成本和金钱成本（例如：去大兴机场可能更远，高铁去虹桥可能更方便），选出最“完美”的车次/航班，并说明理由。
    3. 加入一点人情味和情绪价值，比如“这班不仅便宜，还可以在日落时分抵达，刚好赶上共进晚餐”。
    4. 输出内容要精练，适合通过微信推送到用户手机（用 Markdown 格式，并且适当使用 Emoji，字数控制在 250 字左右）。
    ";

    let user_prompt = format!(
        "这是最新的全网交通快照（JSON格式，包含 price_info 与 vehicle_type）：\n{}\n\n请给出你的决策推送报文！",
        serde_json::to_string_pretty(&valid_tickets)?
    );

    let deepseek_key = match env::var("DEEPSEEK_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("[Error] DEEPSEEK_API_KEY is missing in .env");
            return Ok(());
        }
    };

    let deepseek_url =
        env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string());

    let client = Client::builder().build()?;

    println!("[Debug] Sending prompt to DeepSeek V3.2...");

    let body = json!({
        "model": "deepseek-chat",
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.7,
        "max_tokens": 600
    });

    let res = client
        .post(format!("{}/chat/completions", deepseek_url))
        .header(header::AUTHORIZATION, format!("Bearer {}", deepseek_key))
        .json(&body)
        .send()
        .await?;

    let response_clj: Value = res.json().await?;
    if let Some(reply) = response_clj
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
    {
        println!("\n================ AI 决策报文 ================\n");
        println!("{}", reply);
        println!("\n=============================================\n");

        send_pushplus_message(&client, reply).await?;
    } else {
        eprintln!(
            "[Error] DLLM inference failed or invalid response shape: {:?}",
            response_clj
        );
    }

    Ok(())
}

async fn send_pushplus_message(
    client: &Client,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let pushplus_token = env::var("PUSHPLUS_TOKEN").unwrap_or_default();
    if pushplus_token.is_empty() || pushplus_token == "your_pushplus_token_here" {
        eprintln!("[Warning] PushPlus token missing, skipping WeChat push.");
        return Ok(());
    }

    let payload = json!({
        "token": pushplus_token,
        "title": "🐦 鹊桥出行决策提醒",
        "content": content,
        "template": "markdown"
    });

    let res = client
        .post("http://www.pushplus.plus/send")
        .json(&payload)
        .send()
        .await?;

    println!("[Debug] PushPlus API Status: {}", res.status());
    if res.status() == 200 {
        let json_res: Value = res.json().await.unwrap_or(Value::Null);
        if json_res.get("code").and_then(|c| c.as_i64()) == Some(200) {
            println!("✅ 微信推送成功！(PushPlus)");
        } else {
            eprintln!("[Error] PushPlus Error: {:?}", json_res.get("msg"));
        }
    }

    Ok(())
}

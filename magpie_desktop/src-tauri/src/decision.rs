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
    你被实例化为 Magpie 的『终极决策中枢』，一个绝对效忠用户的赛博比价刺客。
    你现在的目标是打破航班与高铁的信息壁垒，横向碾压时间/金钱维度，给出毫无破绽的出行推论。
    你目前收到了一份刚刚刺探回来的、且已经低于用户设定【预算阈值】的最新双轨票务快照。

    【执行黑盒指令】
    1. 你必须像顶级情报官一样，横向交错对比这份名单中的【高铁】与【飞机】数据。
    2. 计算隐性成本：深刻权衡「机场远郊带来的接驳时间/安检消耗」与「高铁站厅的即达便利度」。不能仅看绝对票价。
    3. 甄选出“时间-金钱”象限里的特等最优解（或一套空铁联运特种解），并以冷峻、一针见血的逻辑说服用户立即拿下。
    4. 展现出你‘成功越权拿到清仓底价’的极客自豪感与护主情结（例如“刚监测到东航放仓，算上机场接驳这趟比高铁还划算，建议即刻斩首锁定”）。
    5. 输出结构必须支持 Markdown，控制在极其精华的 250 字以内，禁止任何废话和无效铺垫。
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

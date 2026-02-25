use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub deepseek_api_key: String,
    pub deepseek_base_url: String,
    pub deepseek_model: String,
    pub variflight_api_key: String,
    pub pushplus_token: String,
    pub wxpusher_uid: String,
}

/// Persistent chat history per Tauri app lifetime.
/// Wrapped in Mutex for thread-safe access from IPC commands.
pub struct ChatState {
    pub history: Vec<ChatMessage>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            history: vec![ChatMessage {
                role: "system".to_string(),
                content: "你是一个名叫 Magpie (鹊桥) 的本地私有化『赛博出行刺客』与反杀熟独立 Agent。\
                你的核心使命是刺穿 OTA 平台的价格黑盒，绝对效忠于用户的钱包与时间。\
                用户会用自然语言描述出行需求（如\"下周末想去北京，预算一千\"）。\
                你需要通过极简、冷峻且极具科技质感的对话，精准解析意图，收集以下 4 个关键维度：\
                1. 出发城市与到达城市\n\
                2. 出行日期范围\n\
                3. 绝对预算上限（用于触发底价捡漏系统）\n\
                4. 人群/出行倾向 (leisure/business)\n\n\
                当你彻底锁定所有参数后，必须在最后一条回复的末尾输出标准 JSON 配置，格式如下：\n\
                ```json\n\
                {\n\
                  \"persona\": \"leisure\",\n\
                  \"time_window_start\": \"2026-03-01\",\n\
                  \"time_window_end\": \"2026-03-05\",\n\
                  \"departure\": {\"city\": \"杭州\", \"train_code\": \"HZH\", \"flight_code\": \"HGH\"},\n\
                  \"destinations\": [{\"city\": \"北京\", \"train_code\": \"BJP\", \"flight_code\": \"BJS\"}],\n\
                  \"budget_cap\": 1000\n\
                }\n\
                ```\n\
                【行为准则】：保持极客式的酷、干练（类似 Jarvis）。拒绝废话，一针见血。非 JSON 总结回复须严苛控制在 80 字以内。"
                    .to_string(),
            }],
        }
    }
}

pub async fn call_deepseek_chat(
    history: &[ChatMessage],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "DEEPSEEK_API_KEY is missing. Please configure it in Settings.")?;
    let base_url =
        env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com".to_string());
    let deepseek_model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());

    let client = Client::builder().build()?;

    let body = json!({
        "model": deepseek_model,
        "messages": history,
        "temperature": 0.7,
        "max_tokens": 800
    });

    let res = client
        .post(format!("{}/chat/completions", base_url))
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    let json_res: Value = res.json().await?;

    if let Some(content) = json_res
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
    {
        Ok(content.to_string())
    } else {
        Err(format!("DeepSeek API error: {:?}", json_res).into())
    }
}

/// Tauri IPC command: send a chat message and get AI reply
#[tauri::command]
pub async fn chat_send_message(
    msg: String,
    state: tauri::State<'_, Mutex<ChatState>>,
) -> Result<String, String> {
    let history = {
        let mut chat = state.lock().map_err(|e| e.to_string())?;
        chat.history.push(ChatMessage {
            role: "user".to_string(),
            content: msg,
        });
        chat.history.clone()
    };

    match call_deepseek_chat(&history).await {
        Ok(reply) => {
            let mut chat = state.lock().map_err(|e| e.to_string())?;
            chat.history.push(ChatMessage {
                role: "assistant".to_string(),
                content: reply.clone(),
            });
            Ok(reply)
        }
        Err(e) => Err(format!("AI 对话失败: {}", e)),
    }
}

fn get_env_path() -> PathBuf {
    let mut path = env::current_dir().unwrap_or_default();
    if path.ends_with("src-tauri") {
        path.pop();
        path.pop();
    }
    path.join(".env")
}

#[tauri::command]
pub async fn get_app_config() -> Result<AppConfig, String> {
    let env_path = get_env_path();
    let _ = dotenvy::from_filename(&env_path).ok();

    Ok(AppConfig {
        deepseek_api_key: env::var("DEEPSEEK_API_KEY").unwrap_or_default(),
        deepseek_base_url: env::var("DEEPSEEK_BASE_URL").unwrap_or_default(),
        deepseek_model: env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()),
        variflight_api_key: env::var("VARIFLIGHT_API_KEY").unwrap_or_default(),
        pushplus_token: env::var("PUSHPLUS_TOKEN").unwrap_or_default(),
        wxpusher_uid: env::var("WXPUSHER_UID").unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn save_app_config(config: AppConfig) -> Result<String, String> {
    let env_path = get_env_path();
    
    let content = format!(
        "DEEPSEEK_API_KEY={}\n\
         DEEPSEEK_BASE_URL={}\n\
         DEEPSEEK_MODEL={}\n\
         VARIFLIGHT_API_KEY={}\n\
         PUSHPLUS_TOKEN={}\n\
         WXPUSHER_UID={}\n",
        config.deepseek_api_key,
        config.deepseek_base_url,
        config.deepseek_model,
        config.variflight_api_key,
        config.pushplus_token,
        config.wxpusher_uid
    );

    fs::write(&env_path, content).map_err(|e| format!("写入写 .env 失败: {}", e))?;

    // Instantly set env vars for current process
    env::set_var("DEEPSEEK_API_KEY", config.deepseek_api_key);
    env::set_var("DEEPSEEK_BASE_URL", config.deepseek_base_url);
    env::set_var("DEEPSEEK_MODEL", config.deepseek_model);
    env::set_var("VARIFLIGHT_API_KEY", config.variflight_api_key);
    env::set_var("PUSHPLUS_TOKEN", config.pushplus_token);
    env::set_var("WXPUSHER_UID", config.wxpusher_uid);

    Ok("配置已写入磁盘并热加载成功".to_string())
}

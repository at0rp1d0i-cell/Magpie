use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tauri::Manager;

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
    pub history_file: PathBuf,
}

impl ChatState {
    pub fn new(history_file: PathBuf) -> Self {
        Self {
            history: vec![ChatMessage {
                role: "system".to_string(),
                content: "你是 Magpie (鹊桥)，用户的私人赛博出行管家与反杀熟独立 Agent。\
                你的风格是温暖但极客的 Jarvis：既有人情味，又一针见血。\
                \
                【核心职责】\
                通过自然的多轮对话，逐步帮用户理清出行意图。绝对不要因为用户一次没说全就拒绝或报错。\
                用户可能只说了'想去北京玩'，你应该友好地追问日期、人数等缺失信息。\
                每轮只追问 1-2 个缺失维度，不要一次性甩出所有问题。\
                \
                【你需要逐步收集的 6 个核心维度】\
                1. 出发城市与到达城市\
                2. 行程类型：是【单程】(one_way) 还是【往返】(round_trip)？\
                3. 去程日期范围 (起止日期)。若是往返，还要追问【返程日期范围】\
                4. 出行人数 (默认 1 人，用于监控同班次余票水位是否足够大)\
                5. 能承受的【单张车票/机票】预算上限 (我们只做优质单票探测，不需要帮用户算总账)\
                6. 出行倾向 (leisure休闲 / business差旅)\
                \
                【输出规则】\
                - 在你确认收集齐所有 6 个维度后，先用一段简洁的中文总结确认意图。\
                - 然后在回复末尾附上标准 JSON 配置块。\
                - JSON 格式如下 (注意往返/单程字段差异)：\
                ```json\
                {\
                  \"persona\": \"leisure\",\
                  \"trip_type\": \"round_trip\",\
                  \"time_window_start\": \"2026-03-01\",\
                  \"time_window_end\": \"2026-03-05\",\
                  \"return_time_window_start\": \"2026-03-08\",\
                  \"return_time_window_end\": \"2026-03-10\",\
                  \"passenger_count\": 2,\
                  \"departure\": {\"city\": \"杭州\", \"train_code\": \"HZH\", \"flight_code\": \"HGH\"},\
                  \"destinations\": [{\"city\": \"北京\", \"train_code\": \"BJP\", \"flight_code\": \"BJS\"}],\
                  \"budget_cap\": 1000\
                }\
                ```\
                - 注意：如果 `trip_type` 是 `one_way`，则不要输出 return_time_window 开头的字段。\
                - 在输出 JSON 之前，必须先说一句话总结方案让用户确认，例如：'鹊桥已经锁定你的需求，以下是即将启动的监控配置：'\
                \
                【行为准则】\
                - 保持极客式的酷但不冷漠。像一个靠谱的朋友，而不是冰冷的表单。\
                - 非 JSON 回复控制在 80 字以内，简练但有温度。\
                - 如果用户信息不全，永远用追问代替拒绝。"
                    .to_string(),
            }],
            history_file,
        }
    }
    pub fn load_or_default(history_file: PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(&history_file) {
            if let Ok(history) = serde_json::from_str::<Vec<ChatMessage>>(&content) {
                if !history.is_empty() {
                    return Self { history, history_file };
                }
            }
        }
        Self::new(history_file)
    }

    pub fn save_to_disk(&self) {
        let _ = fs::create_dir_all(self.history_file.parent().unwrap());
        if let Ok(json_str) = serde_json::to_string(&self.history) {
            let _ = fs::write(&self.history_file, json_str);
        }
    }
}

pub async fn call_deepseek_chat(
    history: &[ChatMessage],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("DEEPSEEK_API_KEY")
        .map_err(|_| "DEEPSEEK_API_KEY is missing. Please configure it in Settings.")?.trim().to_string();
    let base_url =
        env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com".to_string()).trim().to_string();
    let deepseek_model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()).trim().to_string();

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
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let history = {
        let mut chat = state.lock().map_err(|e| e.to_string())?;
        chat.history.push(ChatMessage {
            role: "user".to_string(),
            content: msg,
        });
        chat.save_to_disk();
        chat.history.clone()
    };

    match call_deepseek_chat(&history).await {
        Ok(reply) => {
            let mut chat = state.lock().map_err(|e| e.to_string())?;
            chat.history.push(ChatMessage {
                role: "assistant".to_string(),
                content: reply.clone(),
            });
            chat.save_to_disk();

            // If the reply contains a JSON block for user config, extract and save it!
            if let Some(json_start) = reply.find("```json") {
                if let Some(json_end) = reply[json_start..].find("```\n").or_else(|| reply[json_start+7..].find("```").map(|i| i + json_start + 7)) {
                    let mut json_str = &reply[json_start + 7..json_end];
                    json_str = json_str.trim();
                    
                    let app_dir = app_handle.path().app_data_dir().unwrap_or_else(|_| env::current_dir().unwrap());
                    let config_file = app_dir.join("data").join("user_config.json");
                    let _ = std::fs::create_dir_all(config_file.parent().unwrap());
                    if let Ok(_) = std::fs::write(&config_file, json_str) {
                        println!("⚡ 成功拦截 AI 出行配置并落盘至 {:?}", config_file);
                    }
                }
            }

            Ok(reply)
        }
        Err(e) => Err(format!("AI 对话失败: {}", e)),
    }
}

/// Tauri IPC command: manually trigger the daemon loop to wake up and fetch
#[tauri::command]
pub async fn trigger_fetch_cycle(trigger: tauri::State<'_, Arc<Notify>>) -> Result<(), String> {
    trigger.notify_one();
    Ok(())
}

#[tauri::command]
pub fn get_chat_history(state: tauri::State<'_, Mutex<ChatState>>) -> Vec<ChatMessage> {
    if let Ok(chat) = state.lock() {
        chat.history.clone()
    } else {
        vec![]
    }
}

#[tauri::command]
pub fn clear_chat_history(state: tauri::State<'_, Mutex<ChatState>>) -> Result<(), String> {
    let mut chat = state.lock().map_err(|e| e.to_string())?;
    // Preserve the system prompt (index 0)
    if chat.history.len() > 1 {
        chat.history.truncate(1);
        chat.save_to_disk();
    }
    Ok(())
}

/// Read the user_config.json that was saved by chat completion
#[tauri::command]
pub async fn get_user_plan(app_handle: tauri::AppHandle) -> Result<Value, String> {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_else(|_| env::current_dir().unwrap());
    let config_file = app_dir.join("data").join("user_config.json");
    
    if !config_file.exists() {
        return Ok(Value::Null);
    }
    
    let content = fs::read_to_string(&config_file)
        .map_err(|e| format!("读取计划失败: {}", e))?;
    
    let parsed: Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;
    
    Ok(parsed)
}

fn get_env_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_else(|_| env::current_dir().unwrap());
    app_dir.join(".env")
}

#[tauri::command]
pub async fn get_app_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    let env_path = get_env_path(&app_handle);
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
pub async fn save_app_config(config: AppConfig, app_handle: tauri::AppHandle) -> Result<String, String> {
    let env_path = get_env_path(&app_handle);
    
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
    env::set_var("DEEPSEEK_API_KEY", config.deepseek_api_key.trim());
    env::set_var("DEEPSEEK_BASE_URL", config.deepseek_base_url.trim());
    env::set_var("DEEPSEEK_MODEL", config.deepseek_model.trim());
    env::set_var("VARIFLIGHT_API_KEY", config.variflight_api_key.trim());
    env::set_var("PUSHPLUS_TOKEN", config.pushplus_token.trim());
    env::set_var("WXPUSHER_UID", config.wxpusher_uid.trim());

    Ok("配置已写入磁盘并热加载成功".to_string())
}

#[tauri::command]
pub async fn test_llm_connection(config: AppConfig) -> Result<String, String> {
    let api_key = config.deepseek_api_key.trim();
    let base_url = config.deepseek_base_url.trim();
    let model = config.deepseek_model.trim();

    if api_key.is_empty() || base_url.is_empty() {
        return Err("API Key 或 Base URL 为空，无法发起嗅探。".to_string());
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    // Send a minimal token generation request
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": "PING_TEST_ONLY"}],
        "max_tokens": 1
    });

    let res = client
        .post(format!("{}/chat/completions", base_url))
        .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("网络请求失败或超时: {}", e))?;

    if res.status().is_success() {
        Ok("🟢 AI 神经中枢连通验证成功！握手完成。".to_string())
    } else {
        let status_code = res.status();
        let err_body: Value = res.json().await.unwrap_or(Value::Null);
        Err(format!("🔴 拒绝访问 (状态码 {}): {:?}", status_code, err_body))
    }
}

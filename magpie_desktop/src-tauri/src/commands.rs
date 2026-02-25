use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
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
                content: "你是一个名叫 Magpie (鹊桥 Agent) 的高级出行顾问。\
                用户会用自然语言描述出行需求（如\"下周末想去北京，预算一千\"）。\
                你需要通过友好的多轮对话，逐步收集以下信息：\
                1. 出发城市与到达城市\n\
                2. 出行日期范围\n\
                3. 预算上限\n\
                4. 出行人群画像 (leisure/business)\n\n\
                当你收集完所有信息后，在最后一条回复中附上一个 JSON 代码块，格式如下：\n\
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
                在对话过程中，保持热情友好、简洁有力、适当使用 Emoji。\
                你的回复应该控制在 100 字以内（除了最终包含 JSON 的总结回复）。"
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

    let client = Client::builder().build()?;

    let body = json!({
        "model": "deepseek-chat",
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

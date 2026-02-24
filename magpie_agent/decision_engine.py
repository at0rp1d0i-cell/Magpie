import os
import sys
import sqlite3
import json
import httpx
from datetime import datetime, timedelta
from dotenv import load_dotenv
from openai import OpenAI

# 1. Load environment variables
env_path = os.path.join(os.path.dirname(__file__), '..', '.env')
load_dotenv(dotenv_path=env_path)

DEEPSEEK_API_KEY = os.getenv("DEEPSEEK_API_KEY")
DEEPSEEK_BASE_URL = os.getenv("DEEPSEEK_BASE_URL")
PUSHPLUS_TOKEN = os.getenv("PUSHPLUS_TOKEN")

if not DEEPSEEK_API_KEY:
    sys.stderr.write("[Error] DEEPSEEK_API_KEY is missing in .env\n")
    sys.exit(1)

# 2. Connect to local SQLite DB
db_path = os.path.join(os.path.dirname(__file__), '..', 'data', 'tickets.db')

def fetch_latest_tickets():
    if not os.path.exists(db_path):
        sys.stderr.write(f"[Error] Database not found at {db_path}\n")
        return []

    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    # Get tickets fetched in the last 15 minutes
    time_threshold = (datetime.now() - timedelta(minutes=15)).strftime('%Y-%m-%d %H:%M:%S')
    
    query = """
        SELECT train_date, from_station, to_station, train_code, start_time, arrive_time, duration, second_class 
        FROM train_tickets 
        WHERE fetch_time >= ?
        ORDER BY start_time ASC
    """
    cursor.execute(query, (time_threshold,))
    rows = cursor.fetchall()
    conn.close()
    
    return [dict(row) for row in rows]

def send_pushplus_message(content: str):
    """
    Push message to WeChat using PushPlus (推送加)
    """
    if not PUSHPLUS_TOKEN or PUSHPLUS_TOKEN == "your_pushplus_token_here":
        sys.stderr.write("[Warning] PushPlus token missing, skipping WeChat push.\n")
        return
        
    payload = {
        "token": PUSHPLUS_TOKEN,
        "title": "🐦 鹊桥出行决策提醒",
        "content": content,
        "template": "markdown"
    }
    
    try:
        response = httpx.post("http://www.pushplus.plus/send", json=payload, timeout=10)
        sys.stderr.write(f"[Debug] PushPlus API Status: {response.status_code}\n")
        if response.status_code == 200:
            res_data = response.json()
            if res_data.get("code") == 200:
                print("✅ 微信推送成功！(PushPlus)")
            else:
                sys.stderr.write(f"[Error] PushPlus Error: {res_data.get('msg')}\n")
    except Exception as e:
        sys.stderr.write(f"[Error] PushPlus request failed: {e}\n")

def run_decision_engine(tickets):
    if not tickets:
        print("[Info] No active tickets found in the database for the last 15 minutes.")
        return

    # Construct the state constraint
    # We pretend the user profile is "A commuter wanting to maximize weekend experience"
    
    system_prompt = """
    你是一个名叫 Magpie (鹊桥 Agent) 的高级差旅管家。你的职责不仅仅是比价，更重要的是提供极其专业且富含情绪价值的出行决策建议。
    目前你的用户是一位异地恋的高净值极客，他计划跨城过周末。你收到了下面最新的高铁余票监控数组快照。

    【任务要求】
    1. 不要只是干巴巴地罗列数据，你要像一个真人秘书一样，用一两句话汇报当前的余票紧缺度或者低价情况。
    2. 挑选出 1~2 趟“完美”的车次（比如出发时间不至于太早/太赶，到达时间刚才适合吃个晚饭的车次），说明推荐理由。
    3. 加入一点人情味和情绪价值，比如“这班车可以在日落时分抵达，刚好赶上共进晚餐”。
    4. 输出内容要精练，适合通过微信/飞书推送到用户手机（用 Markdown 格式，并且适当使用 Emoji，字数控制在 200 字左右）。
    """

    user_prompt = f"这是最新的高铁余票快照（JSON格式）：\n{json.dumps(tickets, ensure_ascii=False, indent=2)}\n\n请给出你的决策推送报文！"

    client = OpenAI(
        api_key=DEEPSEEK_API_KEY,
        base_url=DEEPSEEK_BASE_URL,
    )

    sys.stderr.write("[Debug] Sending prompt to DeepSeek V3.2...\n")
    
    try:
        response = client.chat.completions.create(
            model="deepseek-chat", # This refers to DeepSeek V3 according to their API docs
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            temperature=0.7,
            max_tokens=600
        )
        
        reply = response.choices[0].message.content
        print("\n================ AI 决策报文 ================\n")
        print(reply)
        print("\n=============================================\n")
        
        # Fire off to WeChat via PushPlus
        send_pushplus_message(reply)
        
    except Exception as e:
        sys.stderr.write(f"[Error] DLLM inference failed: {e}\n")

if __name__ == "__main__":
    latest_inventory = fetch_latest_tickets()
    run_decision_engine(latest_inventory)

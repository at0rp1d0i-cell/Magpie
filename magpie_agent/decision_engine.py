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
        SELECT travel_date, from_station_name, to_station_name, vehicle_code, vehicle_type, start_time, arrive_time, duration, price_info 
        FROM omni_tickets 
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

def get_budget_cap():
    """Reads budget_cap from generated user_config.json if available"""
    config_path = os.path.join(os.path.dirname(__file__), '..', 'data', 'user_config.json')
    try:
        with open(config_path, 'r', encoding='utf-8') as f:
            config = json.load(f)
            return config.get("budget_cap", 9999)
    except FileNotFoundError:
        return 9999
    except json.JSONDecodeError:
        return 9999

def run_decision_engine(tickets):
    if not tickets:
        print("[Info] No active tickets found in the database for the last 15 minutes.")
        return

    budget_cap = get_budget_cap()
    
    # Pre-filter tickets exceeding budget_cap to avoid wasting LLM tokens and causing invalid pushes
    valid_tickets = []
    for t in tickets:
        try:
            price_str = t.get("price_info", "")
            # Price info might be complex now: '二等座:￥100|一等座:￥200|无座:￥100' or '￥500' for flights
            # For interception, we attempt to find any price <= budget_cap
            import re
            prices = re.findall(r'￥(\d+(?:\.\d+)?)', price_str)
            if prices:
                if any(float(p) <= budget_cap for p in prices):
                    valid_tickets.append(t)
            else:
                valid_tickets.append(t) # include ones we can't parse
        except ValueError:
            valid_tickets.append(t) # include ones we can't parse just in case
            
    if not valid_tickets:
        print(f"🔕 拦截机制触发: 监控到 {len(tickets)} 条空铁数据，但没有符合心理预算 (≤￥{budget_cap}) 的可行方案，拦截推送。")
        return
        
    print(f"✅ 从 {len(tickets)} 条数据中筛出 {len(valid_tickets)} 条低于 ￥{budget_cap} 预算的三维时空数据送往 LLM 决策...")

    # Construct the state constraint
    # We pretend the user profile is "A commuter wanting to maximize weekend experience"
    
    system_prompt = """
    你是一个名叫 Magpie (鹊桥 Agent) 的高级差旅管家。你的职责不仅仅是比价，更重要的是提供极其专业且富含情绪价值的出行决策建议。
    目前你的用户是一位异地恋的高净值极客，他计划跨城过周末。你收到了下面最新的包含【高铁、飞机】的双轨余票监控快照。

    【任务要求】
    1. 你需要进行“空铁联合决策”，像一个真人秘书一样汇报当前的余票紧缺度或者低价情况。
    2. 如果高铁和飞机同在一个时间段，对比它们的时间成本和金钱成本（例如：去大兴机场可能更远，高铁去虹桥可能更方便），选出最“完美”的车次/航班，并说明理由。
    3. 加入一点人情味和情绪价值，比如“这班不仅便宜，还可以在日落时分抵达，刚好赶上共进晚餐”。
    4. 输出内容要精练，适合通过微信推送到用户手机（用 Markdown 格式，并且适当使用 Emoji，字数控制在 250 字左右）。
    """

    user_prompt = f"这是最新的全网交通快照（JSON格式，包含 price_info 与 vehicle_type）：\n{json.dumps(tickets, ensure_ascii=False, indent=2)}\n\n请给出你的决策推送报文！"

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

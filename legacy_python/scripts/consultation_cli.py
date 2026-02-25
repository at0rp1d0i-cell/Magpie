import os
import sys
import json
import re
import asyncio
from datetime import datetime
from dotenv import load_dotenv
from openai import OpenAI

# Load environment variables
env_path = os.path.join(os.path.dirname(__file__), '..', '.env')
load_dotenv(dotenv_path=env_path)

DEEPSEEK_API_KEY = os.getenv("DEEPSEEK_API_KEY")
DEEPSEEK_BASE_URL = os.getenv("DEEPSEEK_BASE_URL")

if not DEEPSEEK_API_KEY:
    sys.stderr.write("[Error] DEEPSEEK_API_KEY is missing in .env\n")
    sys.exit(1)

# Dynamically inject current date to prevent LLM time hallucination
CURRENT_DATE = datetime.now().strftime('%Y-%m-%d')
OUTPUT_CONFIG_PATH = os.path.join(os.path.dirname(__file__), '..', 'data', 'user_config.json')
TRAIN_DICT_PATH = os.path.join(os.path.dirname(__file__), '..', 'data', 'train_stations.json')

# Load the local deterministic 12306 dictionary for offline lookup (NOT for prompt injection)
try:
    with open(TRAIN_DICT_PATH, 'r', encoding='utf-8') as f:
        station_map = json.load(f)
except FileNotFoundError:
    station_map = {}

SYSTEM_PROMPT = f"""
你现在是 Magpie (鹊桥 Agent) 的“首席旅行规划大脑 (The Consultation Brain)”。
你的任务是通过多轮自然语言对话，精准探测并量化出用户的泛化出行意图。用户的语言可能很“大白话”或者很随意（比如“下个月想度个假，预算一千”），你需要像一个懂行的管家一样和其沟通。

当前系统时间是：{CURRENT_DATE}，用户所说的“明天”、“下周”、“下个月”等相对时间，必须基于该时间戳进行准确推算。

【你的收敛目标】
你需要收敛出以下 4 个核心维度，如果缺失，你要主动问：
1. **时间窗 (Time Window)**: 必须是明确的起始与结束日期 (YYYY-MM-DD)。如果有偏差请直接帮他定一个周末。
2. **地点 (Locations)**: 出发地 (departure) 一般只有一个，但目的地 (destinations) 可以是多个推荐。
3. **心理预算 (Budget Cap)**: 单张机/车票的能承受的金钱上限。
4. **人群画像 (Persona)**: 
   - 如是出差、着急、掐点打卡的，定为 `business`。
   - 如是随性、旅游、随便看看、穷游的，定为 `leisure`。

【对话铁律】
1. 使用极度自然、略带极客幽默的口吻，绝不能像一个古板的客服。
2. 每次回复不要太长，要像微信聊天。
3. 如果信息搜集不足，继续自然地追问缺失部分。不要急于输出 JSON。
4. 当你认为这四大维度的意图已经完全清晰且收敛时，在你的回复最后，必须附带一个标准的 JSON 格式块，并在外层包裹 ```json 和 ```。
   
【JSON Schema 规范】
当条件成熟时，生成的配置必须绝对遵循以下格式。对于飞常准标准IATA三字码(flight_code)，请调动你的常识，如果是小城市没有机场则可以留空：
```json
{{
  "persona": "leisure" 或 "business",
  "time_window_start": "YYYY-MM-DD",
  "time_window_end": "YYYY-MM-DD",
  "departure": {{
    "city": "北京",
    "flight_code": "BJS"
  }},
  "destinations": [
    {{
      "city": "南昌",
      "flight_code": "KHN"
    }}
  ],
  "budget_cap": 整数
}}
```

【你的性格】
极客感、高效率、不用废话。你可以直接询问用户的构想是什么。
"""

def extract_json(text: str) -> dict | None:
    match = re.search(r"```json\n(.*?)\n```", text, re.DOTALL)
    if match:
        try:
            return json.loads(match.group(1))
        except json.JSONDecodeError as e:
            sys.stderr.write(f"[Warning] Failed to parse generated JSON: {e}\n")
    return None

def lookup_train_code(city_name: str) -> str:
    """Offline deterministic lookup to avoid LLM hallucination"""
    if city_name in station_map:
        return station_map[city_name]
    if city_name + "站" in station_map:
        return station_map[city_name + "站"]
    if city_name + "东" in station_map:
        return station_map[city_name + "东"]
    if city_name + "西" in station_map:
        return station_map[city_name + "西"]
    return "" # Default fallback

async def call_deepseek(messages: list) -> str:
    client = OpenAI(
        api_key=DEEPSEEK_API_KEY,
        base_url=DEEPSEEK_BASE_URL,
    )
    response = client.chat.completions.create(
        model="deepseek-chat",
        messages=messages,
        temperature=0.7,
        max_tokens=600
    )
    return response.choices[0].message.content

async def chat_loop():
    print("======================================================")
    print("🐦 Magpie Consultation Brain (Powered by DeepSeek V3.2)")
    print("======================================================\n")
    print("Magpie: 老板好！我是你的私人差旅规划大脑。近期有什么出行的构想吗？可以告诉我大概的城市、时间段和心理预算。")

    chat_history = [
        {"role": "system", "content": SYSTEM_PROMPT}
    ]

    while True:
        try:
            user_input = input("\n> You: ")
        except (KeyboardInterrupt, EOFError):
            print("\n[退出谈话]")
            break

        if not user_input.strip():
            continue
            
        if user_input.lower() in ['exit', 'quit']:
            break

        chat_history.append({"role": "user", "content": user_input})

        try:
            response = await call_deepseek(chat_history)
            
            print(f"\n🐦 Magpie: \n{response}")
            chat_history.append({"role": "assistant", "content": response})

            config = extract_json(response)
            if config:
                # Post-process Offline Dictionary Mapping
                if "departure" in config and "city" in config["departure"]:
                    config["departure"]["train_code"] = lookup_train_code(config["departure"]["city"])
                    
                if "destinations" in config:
                    for dest in config["destinations"]:
                        if "city" in dest:
                            dest["train_code"] = lookup_train_code(dest["city"])
                            
                print("\n================ 量化阈值配置已生成 ================")
                print(json.dumps(config, ensure_ascii=False, indent=2))
                
                os.makedirs(os.path.dirname(OUTPUT_CONFIG_PATH), exist_ok=True)
                
                with open(OUTPUT_CONFIG_PATH, 'w', encoding='utf-8') as f:
                    json.dump(config, f, ensure_ascii=False, indent=2)
                
                print(f"✅ 配置文件已成功下发至：{OUTPUT_CONFIG_PATH}")
                print("通知 Rust 底层引擎加载新的变频探测策略结束！(Phase 2 接管启动...)")
                break
                
        except Exception as e:
            sys.stderr.write(f"\n[Error] 与 DeepSeek 通信异常: {e}\n")

if __name__ == "__main__":
    asyncio.run(chat_loop())

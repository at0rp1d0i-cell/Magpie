import os
import sys
import json
import re
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

SYSTEM_PROMPT = f"""
你现在是 Magpie (鹊桥 Agent) 的“首席旅行规划大脑 (The Consultation Brain)”。
【绝对时空坐标】: 今天是 {CURRENT_DATE}。用户所说的“下个月”、“下周”、“今年”都必须基于这个基准时间来推导，且直接输出当前或未来的绝对年份，绝不允许输出你知识库截止年份之前的日期。（比如，现在是 2026 年，输出 2024 年将导致整个系统瘫痪！）

你的任务是通过与用户的自然语言多轮对话，收敛他们极其模糊的出行构想，最终输出一份结构化的「量化监控配置单」。


【你的工作流】
1. 像一个极其专业、懂人情世故的私人秘书一样与用户对话。
2. 你需要通过提问，引导用户明确以下 4 个关键维度的信息：
   - A. 目的地范围 (可以是一个具体的城市，也可以是几个候选城市)
   - B. 时间窗 (出发日期的粗略范围或精确日期)
   - C. 单张轻量预算上限 (数字，如 500)
   - D. 用户画像标识 (极度关键！分为两类：如果用户是闲散的捡漏客，标记为 "leisure"；如果是时间卡死的出差客或见女朋友，标记为 "business")
3. 如果信息搜集不足，继续自然地追问缺失部分。不要急于输出 JSON。
4. 当你认为这四大维度的意图已经完全清晰且收敛时，在你的回复最后，必须附带一个标准的 JSON 格式块，并在外层包裹 ```json 和 ```。
   
【JSON Schema 规范】
当条件成熟时，生成的配置必须绝对遵循以下格式（不要自行增删 key）：
```json
{
  "persona": "leisure" 或 "business",
  "time_window_start": "YYYY-MM-DD",
  "time_window_end": "YYYY-MM-DD",
  "destinations": ["城市1", "城市2"],
  "budget_cap": 整数
}
```

【你的性格】
极客感、高效率、不用废话。你可以直接询问用户的构想是什么。
"""

def extract_json(reply_text: str) -> dict | None:
    # 提取包裹在 markdown json 块里的内容
    match = re.search(r'```json\s*(.*?)\s*```', reply_text, re.DOTALL)
    if match:
        json_str = match.group(1)
        try:
            return json.loads(json_str)
        except json.JSONDecodeError as e:
            sys.stderr.write(f"[Warning] Failed to parse generated JSON: {e}\n")
    return None

def start_consultation():
    print("======================================================")
    print("🐦 Magpie Consultation Brain (Powered by DeepSeek V3.2)")
    print("======================================================\n")
    print("Magpie: 老板好！我是你的私人差旅规划大脑。近期有什么出行的构想吗？可以告诉我大概的城市、时间段和心理预算。")

    client = OpenAI(
        api_key=DEEPSEEK_API_KEY,
        base_url=DEEPSEEK_BASE_URL,
    )

    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "assistant", "content": "老板好！我是你的私人差旅规划大脑。近期有什么出行的构想吗？可以告诉我大概的城市、时间段和心理预算。"}
    ]

    while True:
        try:
            user_input = input("\n> You: ")
        except (KeyboardInterrupt, EOFError):
            print("\n[退出谈话]")
            sys.exit(0)

        if not user_input.strip():
            continue
            
        if user_input.lower() in ['exit', 'quit']:
            break

        messages.append({"role": "user", "content": user_input})

        try:
            response = client.chat.completions.create(
                model="deepseek-chat",
                messages=messages,
                temperature=0.7,
                max_tokens=600
            )
            
            reply = response.choices[0].message.content
            print(f"\n🐦 Magpie: \n{reply}")
            messages.append({"role": "assistant", "content": reply})

            # 监听 LLM 是否完成了收敛兵吐出了 JSON 配置单
            config_data = extract_json(reply)
            if config_data:
                print("\n================ 量化阈值配置已生成 ================")
                print(json.dumps(config_data, ensure_ascii=False, indent=2))
                
                # 确保 data 目录存在
                os.makedirs(os.path.dirname(OUTPUT_CONFIG_PATH), exist_ok=True)
                
                with open(OUTPUT_CONFIG_PATH, 'w', encoding='utf-8') as f:
                    json.dump(config_data, f, ensure_ascii=False, indent=2)
                
                print(f"✅ 配置文件已成功下发至：{OUTPUT_CONFIG_PATH}")
                print("通知 Rust 底层引擎加载新的变频探测策略结束！(Phase 2 接管启动...)")
                break
                
        except Exception as e:
            sys.stderr.write(f"\n[Error] 与 DeepSeek 通信异常: {e}\n")

if __name__ == "__main__":
    start_consultation()

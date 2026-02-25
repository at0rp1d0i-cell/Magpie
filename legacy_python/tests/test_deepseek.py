import os
from openai import OpenAI
from dotenv import load_dotenv
load_dotenv(".env")
client = OpenAI(
    api_key=os.getenv("DEEPSEEK_API_KEY"),
    base_url=os.getenv("DEEPSEEK_BASE_URL"),
)
response = client.chat.completions.create(
    model="deepseek-chat",
    messages=[
        {"role": "user", "content": "请扮演航班查询小助手，给我一个查询北京到南昌明年三月份机票航班信息的JSON Schema，应该包含起飞、降落、价格等字段。"}
    ]
)
print(response.choices[0].message.content)

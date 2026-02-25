import asyncio
import os
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from dotenv import load_dotenv

load_dotenv()

async def test_variflight_itineraries():
    server_params = StdioServerParameters(
        command="npx",
        args=["-y", "@variflight-ai/variflight-mcp"],
        env=dict(os.environ)
    )

    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            
            # 飞常准推荐的测试日期通常不要过于靠后，我们测近期的
            test_date = "2026-03-01"
            print(f"🚀 Testing searchFlightItineraries: BJS to SHA on {test_date} ...")
            try:
                search_result = await session.call_tool(
                    "searchFlightItineraries",
                    arguments={
                        "depCityCode": "BJS",
                        "arrCityCode": "SHA",
                        "depDate": test_date
                    }
                )
                print("\n================ searchFlightItineraries RESULT ================")
                for content in search_result.content:
                    if content.type == "text":
                        print(content.text)
            except Exception as e:
                print(f"❌ Search failed: {str(e)}")

if __name__ == "__main__":
    asyncio.run(test_variflight_itineraries())

import asyncio
import os
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from dotenv import load_dotenv

load_dotenv()

async def test_flight_price():
    server_params = StdioServerParameters(
        command="npx",
        args=["-y", "@variflight-ai/tripmatch-mcp"],
        env=dict(os.environ)
    )

    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            
            # Test getting flight prices for tomorrow
            tomorrow = "2026-02-26"
            print(f"🚀 Testing getFlightPriceByCities: PEK to SHA on {tomorrow} ...")
            try:
                search_result = await session.call_tool(
                    "getFlightPriceByCities",
                    arguments={
                        "dep_city": "PEK",
                        "arr_city": "SHA",
                        "dep_date": tomorrow
                    }
                )
                print("\n================ getFlightPriceByCities RESULT ================")
                for content in search_result.content:
                    if content.type == "text":
                        print(content.text)
            except Exception as e:
                print(f"❌ Search failed: {str(e)}")

if __name__ == "__main__":
    asyncio.run(test_flight_price())

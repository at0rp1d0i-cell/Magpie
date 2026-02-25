import asyncio
import json
import os
from dotenv import load_dotenv
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

load_dotenv()

async def list_tripmatch_tools():
    # Configure the MCP server command and arguments
    server_params = StdioServerParameters(
        command="npx",
        args=["-y", "@variflight-ai/tripmatch-mcp"],
        env=dict(os.environ) # pass VARIFLIGHT_API_KEY from .env
    )

    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            print("✅ Successfully connected to Tripmatch MCP!")
            
            # List all available functions/tools provided by Variflight
            response = await session.list_tools()
            
            print("\n================ AVAILABLE TOOLS ================")
            for tool in response.tools:
                print(f"🔧 Tool: {tool.name}")
                print(f"📝 Desc: {tool.description}")
                print(f"📦 Schema: {json.dumps(tool.inputSchema, indent=2, ensure_ascii=False)}")
                print("-" * 50)
                
            # Optional: Test out the flight search tool
            print("\n🚀 Testing Transfer Search: Beijing(BJS) to Nanchang(KHN) on 2026-03-01 ...")
            try:
                search_result = await session.call_tool(
                    "getFlightAndTrainTransferInfo",
                    arguments={
                        "depcity": "BJS",
                        "arrcity": "KHN",
                        "depdate": "2026-03-01"
                    }
                )
                print("\n================ TEST SEARCH RESULT ================")
                # text content is typically the JSON return
                for content in search_result.content:
                    if content.type == "text":
                        print(content.text)
            except Exception as e:
                print(f"❌ Search failed: {str(e)}")

if __name__ == "__main__":
    asyncio.run(list_tripmatch_tools())

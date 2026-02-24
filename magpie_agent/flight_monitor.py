import sys
import json
import argparse
import asyncio
import os
from typing import List
from dotenv import load_dotenv

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

load_dotenv()

def log(msg):
    sys.stderr.write(msg + "\n")
    sys.stderr.flush()

class MagpieFlightMonitor:
    def __init__(self):
        self.api_key = os.getenv("VARIFLIGHT_API_KEY")
        if not self.api_key:
            log("[Error] VARIFLIGHT_API_KEY is missing in .env")
            sys.exit(1)

    async def execute_query(self, dep_date: str, from_city: str, to_city: str) -> List:
        # Configure the MCP server command
        server_params = StdioServerParameters(
            command="npx",
            args=["-y", "@variflight-ai/variflight-mcp"],
            env=dict(os.environ)
        )

        parsed_inventory = []
        try:
            log("[Debug] 正在拉起 Node.js MCP 守护进程 (Variflight-MCP)...")
            async with stdio_client(server_params) as (read, write):
                async with ClientSession(read, write) as session:
                    await session.initialize()
                    log(f"[Debug] MCP Link Established. Querying Itineraries Info: {from_city} -> {to_city} on {dep_date}")
                    
                    search_result = await session.call_tool(
                        "searchFlightItineraries",
                        arguments={
                            "depCityCode": from_city,
                            "arrCityCode": to_city,
                            "depDate": dep_date
                        }
                    )
                    
                    for content in search_result.content:
                        if content.type == "text":
                            import re
                            text = content.text
                            # Typical text line: "最低价航班为： 航班号：KN5987，起飞时间：2026-03-01 20:55:00，到达时间：2026-03-01 23:15:00，耗时：2h20m，无需中转，超值经济舱价格：490元"
                            pattern = re.compile(r"航班号：([A-Z0-9]+)，起飞时间：([\d\-: ]+)，到达时间：([\d\-: ]+)，耗时：([^，]+)，.*?价格：(\d+)元")
                            matches = pattern.findall(text)
                            
                            for m in matches:
                                flight_node = {
                                    "vehicle_code": m[0],
                                    "vehicle_type": "flight",
                                    "booking_status": "Y", 
                                    "start_time": m[1][11:16],
                                    "arrive_time": m[2][11:16],
                                    "duration": m[3],
                                    "price_info": f"￥{m[4]}", 
                                    "from_station_name": from_city, # Simplification
                                    "to_station_name": to_city
                                }
                                parsed_inventory.append(flight_node)
        except Exception as e:
            log(f"[Error] MCP Query Exception: {str(e)}")
            
        return parsed_inventory

async def main():
    parser = argparse.ArgumentParser(description="Tripmatch MCP Flight Monitor Agent")
    parser.add_argument("--date", required=True, help="Date in YYYY-MM-DD format")
    parser.add_argument("--from_city", required=True, help="From city IATA code (e.g., BJS)")
    parser.add_argument("--to_city", required=True, help="To city IATA code (e.g., NCG)")
    args = parser.parse_args()

    agent = MagpieFlightMonitor()
    inventory = await agent.execute_query(args.date, args.from_city, args.to_city)
    
    # Output MUST be strict JSON to stdout so Rust can parse it
    print(json.dumps(inventory))

if __name__ == "__main__":
    asyncio.run(main())

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
            args=["-y", "@variflight-ai/tripmatch-mcp"],
            env=dict(os.environ)
        )

        parsed_inventory = []
        try:
            log("[Debug] 正在拉起 Node.js MCP 守护进程 (Tripmatch)...")
            async with stdio_client(server_params) as (read, write):
                async with ClientSession(read, write) as session:
                    await session.initialize()
                    log(f"[Debug] MCP Link Established. Querying Transfer Info: {from_city} -> {to_city} on {dep_date}")
                    
                    search_result = await session.call_tool(
                        "getFlightAndTrainTransferInfo",
                        arguments={
                            "depcity": from_city,
                            "arrcity": to_city,
                            "depdate": dep_date
                        }
                    )
                    
                    # Search result content is usually a list of text blocks containing JSON
                    for content in search_result.content:
                        if content.type == "text":
                            try:
                                payload = json.loads(content.text)
                                # The payload structure for getFlightAndTrainTransferInfo:
                                # { "code": 200, "data": [ [ {flight1}, {train1} ], [ {flight2} ] ] }
                                if payload.get("code") == 200 and "data" in payload:
                                    routes = payload["data"]
                                    if not isinstance(routes, list):
                                        continue
                                        
                                    for route_legs in routes:
                                        # For MVP, we only care about direct flights or the first leg if it's a flight
                                        if not route_legs or not isinstance(route_legs, list):
                                            continue
                                            
                                        first_leg = route_legs[0]
                                        if first_leg.get("type") == "航班":
                                            # Normalize to TrainTicket-like structure for Rust compatibility
                                            # We map flight specific fields into a unified schema
                                            flight_node = {
                                                "vehicle_code": first_leg.get("num", ""),
                                                "vehicle_type": "flight",
                                                "booking_status": "Y", # MCP currently doesn't return seat inventory, assume bookable
                                                "start_time": self._format_timestamp(first_leg.get("deptime")),
                                                "arrive_time": self._format_timestamp(first_leg.get("arrtime")),
                                                "duration": self._calculate_duration(first_leg.get("deptime"), first_leg.get("arrtime")),
                                                "price_info": "￥0", # Price is not returned by this specific API, would need getFlightPriceByCities
                                                "from_station_name": first_leg.get("src", ""),
                                                "to_station_name": first_leg.get("dst", "")
                                            }
                                            parsed_inventory.append(flight_node)
                            except json.JSONDecodeError:
                                log(f"[Warning] Failed to parse MCP payload: {content.text[:200]}")
        except Exception as e:
            log(f"[Error] MCP Query Exception: {str(e)}")
            
        return parsed_inventory

    def _format_timestamp(self, ts) -> str:
        if not ts: return ""
        # Tripmatch returns epoch in milliseconds
        from datetime import datetime
        dt = datetime.fromtimestamp(ts / 1000.0)
        return dt.strftime("%H:%M")

    def _calculate_duration(self, dep_ts, arr_ts) -> str:
        if not dep_ts or not arr_ts: return ""
        minutes = int((arr_ts - dep_ts) / 1000 / 60)
        hours = minutes // 60
        mins = minutes % 60
        return f"{hours:02d}:{mins:02d}"

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

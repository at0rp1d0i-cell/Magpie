import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const transport = new StdioClientTransport({
  command: "npx",
  args: ["-y", "@variflight-ai/tripmatch-mcp"],
  env: {
    ...process.env,
    VARIFLIGHT_API_KEY: process.env.VARIFLIGHT_API_KEY
  }
});

const client = new Client(
  {
    name: "magpie-probe",
    version: "1.0.0",
  },
  {
    capabilities: {},
  }
);

async function main() {
  await client.connect(transport);
  console.log("Connected to Tripmatch MCP");

  const tools = await client.listTools();
  console.log("==== AVAILABLE TOOLS ====");
  console.log(JSON.stringify(tools, null, 2));
  
  // Try to search a flight if the tool exists
  const flightTool = tools.tools.find(t => t.name === 'search_flight_with_od');
  if (flightTool) {
      console.log("\n==== TESTING FLIGHT SEARCH ====");
      const result = await client.callTool({
        name: "search_flight_with_od",
        arguments: {
          departure: "北京",
          arrival: "南昌",
          date: "2026-03-01"
        }
      });
      console.log(JSON.stringify(result, null, 2));
  }

  process.exit(0);
}

main().catch(console.error);

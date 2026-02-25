import os
import json
import urllib.request

def fetch_12306_stations():
    print("⏳ Downloading 12306 station names...")
    url = "https://kyfw.12306.cn/otn/resources/js/framework/station_name.js"
    req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
    
    stations = {}
    try:
        with urllib.request.urlopen(req) as response:
            data = response.read().decode('utf-8')
            parts = data.split('@')
            for p in parts:
                if not p.strip() or "station_names" in p:
                    continue
                fields = p.split('|')
                if len(fields) >= 3:
                    name = fields[1]
                    code = fields[2]
                    stations[name] = code
    except Exception as e:
        print(f"❌ Failed to download 12306 data: {e}")
        return

    print(f"✅ Successfully parsed {len(stations)} train stations from 12306.")

    # Save to a dedicated offline lookup table, NEVER inject to LLM Prompt!
    out_path = os.path.join(os.path.dirname(__file__), '..', 'data', 'train_stations.json')
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    
    with open(out_path, 'w', encoding='utf-8') as f:
        json.dump(stations, f, ensure_ascii=False, indent=2)
        
    print(f"✅ Full 12306 offline dictionary saved to {out_path} (Zero API Token cost!)")

if __name__ == "__main__":
    fetch_12306_stations()

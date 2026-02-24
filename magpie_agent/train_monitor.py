import sys
import json
import argparse
from curl_cffi import requests
import time
import re
import urllib.parse
from typing import List

def log(msg):
    sys.stderr.write(msg + "\n")
    sys.stderr.flush()

class MagpieLightweightTrainMonitor:
    def __init__(self):
        self.session = requests.Session(impersonate="chrome110")
        self.session.trust_env = False
        
        try:
            log("[Debug] 请求初始化页面获取网关 Session...")
            self.session.get("https://kyfw.12306.cn/otn/leftTicket/init", timeout=15)
        except Exception as e:
            log(f"[Error] 初始化页面网络异常: {e}")
            
        self._inject_device_fingerprint()

    def _inject_device_fingerprint(self):
        current_timestamp = str(int(time.time() * 1000))
        logdevice_url = "https://kyfw.12306.cn/otn/HttpZF/logdevice"
        params = {
            "algID": "WYEdoc45yu",
            "hashCode": "EhTtj7Znzyie6I21jpgekYReLAnA8fyGEB4VlIGbF0g",
            "FMQw": "0",
            "q4f3": "zh-CN",
            "VPIf": "1",
            "custID": "133",
            "VEek": "unknown",
            "hAqN": "Win32",
            "platform": "WEB",
            "TeRS": "728x1366",
            "tOHY": "24xx768x1366",
            "timestamp": current_timestamp
        }
        
        headers = {
            "Accept": "text/javascript, application/javascript, application/ecmascript, application/x-ecmascript, */*; q=0.01",
            "Referer": "https://kyfw.12306.cn/otn/passport?redirect=/otn/login/conf",
            "Host": "kyfw.12306.cn"
        }
        
        try:
            log(f"[Debug] 请求 logdevice: {logdevice_url}")
            response = self.session.get(logdevice_url, params=params, headers=headers, timeout=15)
            log(f"[Debug] logdevice HTTP Status: {response.status_code}")
            
            pattern = re.compile(r"\('(.*?)'\)")
            match = pattern.findall(response.text)
            
            if match:
                device_telemetry = eval(match[0])
                self.session.cookies.set('RAIL_EXPIRATION', device_telemetry.get('exp', ''), domain='.12306.cn', path='/')
                self.session.cookies.set('RAIL_DEVICEID', device_telemetry.get('dfp', ''), domain='.12306.cn', path='/')
                log("[Log] 成功伪造设备指纹: RAIL_DEVICEID")
        except requests.exceptions.RequestException as e:
            log(f"[Error] 获取指纹网络异常: {e}")

    def execute_query(self, train_date: str, from_telecode: str, to_telecode: str) -> List:
        query_url = "https://kyfw.12306.cn/otn/leftTicket/queryZ"
        
        query_params = {
            "leftTicketDTO.train_date": train_date,
            "leftTicketDTO.from_station": from_telecode,
            "leftTicketDTO.to_station": to_telecode,
            "purpose_codes": "ADULT"
        }

        strict_headers = {
            "Host": "kyfw.12306.cn",
            "Referer": "https://kyfw.12306.cn/otn/leftTicket/init",
            "X-Requested-With": "XMLHttpRequest",
            "Sec-Fetch-Site": "same-origin"
        }

        try:
            log(f"[Debug] 请求查询接口: {query_url}")
            response = self.session.get(query_url, params=query_params, headers=strict_headers, timeout=15)
            log(f"[Debug] query HTTP Status: {response.status_code}")
            if response.status_code == 200:
                try:
                    payload = response.json()
                    if "data" in payload and "result" in payload["data"]:
                        return self._decode_pipe_serialization(payload["data"]["result"], from_telecode, to_telecode)
                except ValueError:
                    log(f"[Warning] JSON Parsing failed, response dump: {response.text[:200]}")
            else:
                log(f"[Warning] HTTP {response.status_code}")
            return []
        except Exception as e:
            log(f"[Error] 查询异常: {e}")
            return []

    def _decode_pipe_serialization(self, raw_result_array: List[str], from_telecode: str, to_telecode: str) -> List:
        parsed_inventory = []
        for raw_train_str in raw_result_array:
            fields = raw_train_str.split('|')
            if len(fields) >= 32:
                train_node = {
                    "vehicle_code": fields[3],
                    "vehicle_type": "train",
                    "booking_status": fields[11], # 'Y' 标识此车次处于售票期
                    "start_time": fields[8],
                    "arrive_time": fields[9],
                    "duration": fields[10],
                    "price_info": f"二等座:{fields[30]}|一等座:{fields[31]}|无座:{fields[26]}",
                    "from_station_name": from_telecode, # Real name requires dictionary map, simplify for MVP
                    "to_station_name": to_telecode
                }
                parsed_inventory.append(train_node)
        return parsed_inventory

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="12306 Train Monitor Agent")
    parser.add_argument("--date", required=True, help="Date in YYYY-MM-DD format")
    parser.add_argument("--from_station", required=True, help="From station telecode (e.g., BJP)")
    parser.add_argument("--to_station", required=True, help="To station telecode (e.g., NCG)")
    args = parser.parse_args()

    agent = MagpieLightweightTrainMonitor()
    inventory = agent.execute_query(args.date, args.from_station, args.to_station)
    
    # Output MUST be strict JSON to stdout so Rust can parse it
    print(json.dumps(inventory))

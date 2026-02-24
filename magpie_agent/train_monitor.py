from curl_cffi import requests
import time
import re
import urllib.parse
from typing import List

class MagpieLightweightTrainMonitor:
    def __init__(self):
        # 强制使用 Session 保持 TCP 长连接池
        # 使用 curl_cffi 伪装真实浏览器 JA3/TLS 指纹
        self.session = requests.Session(impersonate="chrome110")
        self.session.trust_env = False  # Bypassing local proxies like Clash
        
        # 访问 init 页面，获取前端负载均衡 Cookie (JSESSIONID, route, BIGipServer 等)
        try:
            print("[Debug] 请求初始化页面获取网关 Session...")
            self.session.get("https://kyfw.12306.cn/otn/leftTicket/init", timeout=5)
        except Exception as e:
            print(f"[Error] 初始化页面网络异常: {e}")
            
        self._inject_device_fingerprint()

    def _inject_device_fingerprint(self):
        """
        核心突破口：通过静态参数强行请求 logdevice 端点，剥离前端 JS 混淆执行，
        直接榨取 RAIL_DEVICEID (dfp) 和 RAIL_EXPIRATION (exp) 会话凭据。
        """
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
            print(f"[Debug] 请求 logdevice: {logdevice_url}")
            response = self.session.get(logdevice_url, params=params, headers=headers, timeout=5)
            print(f"[Debug] logdevice HTTP Status: {response.status_code}")
            print(f"[Debug] logdevice Response (first 1000 chars): {response.text[:1000]}")
            
            # 服务器下发 JSONP 格式响应，利用正则剥离回调函数外壳
            pattern = re.compile(r"\('(.*?)'\)")
            match = pattern.findall(response.text)
            
            if match:
                # 解析内部 JSON 字符串
                device_telemetry = eval(match[0])
                
                # 将指纹强制植入 Session Cookie 域
                self.session.cookies.set('RAIL_EXPIRATION', device_telemetry.get('exp', ''), domain='.12306.cn', path='/')
                self.session.cookies.set('RAIL_DEVICEID', device_telemetry.get('dfp', ''), domain='.12306.cn', path='/')
                print("[Log] 成功伪造设备指纹: RAIL_DEVICEID")
        except requests.exceptions.RequestException as e:
            print(f"[Error] 获取指纹网络异常: {e}")

    def execute_query(self, train_date: str, from_telecode: str, to_telecode: str) -> List:
        """
        执行余票查询主逻辑，模拟纯净的 AJAX 异步请求。
        """
        # 可能需要根据实际 12306 反爬策略切后缀 query, queryA, queryZ 等
        query_url = "https://kyfw.12306.cn/otn/leftTicket/queryZ"
        
        query_params = {
            "leftTicketDTO.train_date": train_date,
            "leftTicketDTO.from_station": from_telecode,
            "leftTicketDTO.to_station": to_telecode,
            "purpose_codes": "ADULT"
        }

        # 补齐防爬验证强制要求的 Header
        strict_headers = {
            "Host": "kyfw.12306.cn",
            "Referer": "https://kyfw.12306.cn/otn/leftTicket/init",
            "X-Requested-With": "XMLHttpRequest",
            "Sec-Fetch-Site": "same-origin"
        }

        try:
            print(f"[Debug] 请求查询接口: {query_url}")
            response = self.session.get(query_url, params=query_params, headers=strict_headers, timeout=5)
            print(f"[Debug] query HTTP Status: {response.status_code}")
            if response.status_code == 200:
                try:
                    payload = response.json()
                    if "data" in payload and "result" in payload["data"]:
                        return self._decode_pipe_serialization(payload["data"]["result"])
                except ValueError:
                    print(f"[Warning] JSON Parsing failed, response dump: {response.text[:200]}")
            else:
                print(f"[Warning] HTTP {response.status_code}")
            return []
        except Exception as e:
            print(f"[Error] 查询异常: {e}")
            return []

    def _decode_pipe_serialization(self, raw_result_array: List[str]) -> List:
        """
        O(1) 复杂度的高效管道符分割反序列化逻辑。
        """
        parsed_inventory = []
        for raw_train_str in raw_result_array:
            fields = raw_train_str.split('|')
            if len(fields) >= 32:
                train_node = {
                    "train_code": fields[3],
                    "booking_status": fields[11], # 'Y' 标识此车次处于售票期
                    "start_time": fields[8],
                    "arrive_time": fields[9],
                    "duration": fields[10],
                    "second_class": fields[30],   # 高铁二等座
                    "first_class": fields[31],
                    "business_class": fields[32],
                    "no_seat": fields[26]
                }
                parsed_inventory.append(train_node)
        return parsed_inventory

if __name__ == "__main__":
    agent = MagpieLightweightTrainMonitor()
    # BJP (北京) -> NCG (南昌) 
    inventory = agent.execute_query("2026-03-01", "BJP", "NCG")
    
    found_count = 0
    for train in inventory:
        # 宽容匹配：含 O 的或直接数字
        if train['booking_status'] == 'Y' and train['second_class'] not in ['无', '', '*']:
            print(f"[命中] 车次 {train['train_code']} 有二等座! 状态: {train['second_class']} | {train['start_time']}->{train['arrive_time']}")
            found_count += 1
            
    if found_count == 0:
        print("😭 未找到包含二等座有效余票的车次，或者所有车票已抢空。")
    else:
        print(f"✅ 共找到 {found_count} 趟有余票的列车。")

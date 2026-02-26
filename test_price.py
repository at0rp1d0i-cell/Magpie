from curl_cffi import requests
import json
import time
import re
import os

os.environ["HTTP_PROXY"] = ""
os.environ["HTTPS_PROXY"] = ""

def test_price():
    session = requests.Session(impersonate="chrome110")
    
    # 1. Init
    session.get("https://kyfw.12306.cn/otn/leftTicket/init", timeout=15)
    
    # 2. Logdevice
    current_timestamp = str(int(time.time() * 1000))
    logdevice_url = "https://kyfw.12306.cn/otn/HttpZF/logdevice"
    params = {
        "algID": "WYEdoc45yu", "hashCode": "EhTtj7Znzyie6I21jpgekYReLAnA8fyGEB4VlIGbF0g",
        "FMQw": "0", "q4f3": "zh-CN", "VPIf": "1", "custID": "133", "VEek": "unknown",
        "hAqN": "Win32", "platform": "WEB", "TeRS": "728x1366", "tOHY": "24xx768x1366",
        "timestamp": current_timestamp
    }
    headers = {
        "Accept": "text/javascript, application/javascript, */*",
        "Referer": "https://kyfw.12306.cn/otn/passport?redirect=/otn/login/conf",
        "Host": "kyfw.12306.cn", "User-Agent": "Mozilla/5.0"
    }
    
    resp = session.get(logdevice_url, params=params, headers=headers, timeout=15)
    match = re.findall(r"\('(.*?)'\)", resp.text)
    if match:
        dev = eval(match[0])
        session.cookies.set('RAIL_EXPIRATION', dev.get('exp', ''), domain='.12306.cn', path='/')
        session.cookies.set('RAIL_DEVICEID', dev.get('dfp', ''), domain='.12306.cn', path='/')
        print("Device fingerprint set.")
        
    # 3. Query Z
    q_url = "https://kyfw.12306.cn/otn/leftTicket/queryZ"
    q_params = {
        "leftTicketDTO.train_date": "2026-03-01",
        "leftTicketDTO.from_station": "HZH",
        "leftTicketDTO.to_station": "BJP",
        "purpose_codes": "ADULT"
    }
    q_headers = {
        "Host": "kyfw.12306.cn", "Referer": "https://kyfw.12306.cn/otn/leftTicket/init",
        "X-Requested-With": "XMLHttpRequest", "User-Agent": "Mozilla/5.0"
    }
    
    res = session.get(q_url, params=q_params, headers=q_headers)
    data = res.json()["data"]["result"]
    first_train = data[0].split('|')
    train_no = first_train[2]
    train_code = first_train[3]
    seat_types = first_train[35] or first_train[34]
    from_no = first_train[16]
    to_no = first_train[17]
    date = "2026-03-01"
    
    print(f"Train {train_code}: no={train_no}, from={from_no}, to={to_no}, seats={seat_types}")
    
    # 4. Query Price
    # Example: https://kyfw.12306.cn/otn/leftTicket/queryTicketPrice?train_no=5l0000G13061&from_station_no=01&to_station_no=12&seat_types=O9M&train_date=2017-02-05
    p_url = "https://kyfw.12306.cn/otn/leftTicket/queryTicketPrice"
    p_params = {
        "train_no": train_no,
        "from_station_no": from_no,
        "to_station_no": to_no,
        "seat_types": seat_types,
        "train_date": date
    }
    p_res = session.get(p_url, params=p_params, headers=q_headers)
    print("Price HTTP Status:", p_res.status_code)
    print("Price Response:", p_res.text)

test_price()

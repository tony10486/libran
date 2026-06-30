#!/usr/bin/env python3
"""Libran 날씨 위젯 — Open-Meteo API를 호출하여 WOP JSON을 출력합니다."""
import json
import os
import sys
import urllib.request
import urllib.error

# 위젯 디렉토리에서 config 읽기 (또는 하드코딩)
LAT = 37.5665
LON = 126.9780
LOCATION = "서울"

# WMO weather code → 아이콘 + 설명 매핑
WEATHER_MAP = {
    0:  ("☀️", "맑음"),
    1:  ("🌤", "대체로 맑음"),
    2:  ("⛅️", "부분적 흐림"),
    3:  ("☁️", "흐림"),
    45: ("🌫", "안개"),
    48: ("🌫", "서리 안개"),
    51: ("🌦", "약한 이슬비"),
    53: ("🌦", "이슬비"),
    55: ("🌧", "강한 이슬비"),
    56: ("🌧", "약한 어는 이슬비"),
    57: ("🌧", "강한 어는 이슬비"),
    61: ("🌧", "약한 비"),
    63: ("🌧", "비"),
    65: ("🌧", "강한 비"),
    66: ("🌧", "약한 어는 비"),
    67: ("🌧", "강한 어는 비"),
    71: ("🌨", "약한 눈"),
    73: ("🌨", "눈"),
    75: ("❄️", "강한 눈"),
    77: ("❄️", "싸락눈"),
    80: ("🌦", "약한 소나기"),
    81: ("🌦", "소나기"),
    82: ("⛈", "강한 소나기"),
    85: ("🌨", "약한 소나기 눈"),
    86: ("🌨", "강한 소나기 눈"),
    95: ("⛈", "천둥번개"),
    96: ("⛈", "천둥번개 + 우박"),
    99: ("⛈", "강한 천둥번개 + 우박"),
}

URL = (
    f"https://api.open-meteo.com/v1/forecast"
    f"?latitude={LAT}&longitude={LON}"
    f"&current=temperature_2m,apparent_temperature,relative_humidity_2m,"
    f"wind_speed_10m,weathercode,is_day"
    f"&timezone=auto"
)


def main():
    try:
        req = urllib.request.Request(URL, headers={"Accept": "application/json"})
        with urllib.request.urlopen(req, timeout=8) as resp:
            body = resp.read().decode("utf-8")
    except urllib.error.URLError as e:
        output = {
            "version": 1,
            "status": "error",
            "error_message": f"날씨 API 오류: {e.reason}",
            "lines": [{"text": f"❌ 날씨를 불러올 수 없습니다: {e.reason}", "style": "error"}],
        }
        print(json.dumps(output, ensure_ascii=False))
        sys.exit(0)
    except Exception as e:
        output = {
            "version": 1,
            "status": "error",
            "error_message": str(e),
            "lines": [{"text": f"❌ 오류: {e}", "style": "error"}],
        }
        print(json.dumps(output, ensure_ascii=False))
        sys.exit(0)

    try:
        data = json.loads(body)
        cur = data["current"]
        temp = cur["temperature_2m"]
        apparent = cur["apparent_temperature"]
        humidity = cur["relative_humidity_2m"]
        wind = cur["wind_speed_10m"]
        code = cur["weathercode"]
        is_day = cur["is_day"] == 1

        icon, desc = WEATHER_MAP.get(code, ("🌡", "알 수 없음"))
        if not is_day and code == 0:
            icon, desc = "🌙", "맑음(밤)"

        badge = f"{icon} {temp:.0f}°C"

        lines = [
            {
                "text": f"{LOCATION} {temp:.0f}°C {desc}",
                "style": "bold",
                "icon": icon,
            },
            {
                "text": f"체감 온도: {apparent:.0f}°C",
                "style": "normal",
                "icon": "🌡",
            },
            {
                "text": f"습도: {humidity:.0f}%",
                "style": "dim",
                "icon": "💧",
            },
            {
                "text": f"풍속: {wind:.1f} m/s",
                "style": "dim",
                "icon": "💨",
            },
        ]

        output = {
            "version": 1,
            "status": "ok",
            "badge": badge,
            "lines": lines,
            "actions": [
                {
                    "key": "r",
                    "label": "새로고침",
                    "action": "refresh",
                },
            ],
        }
        print(json.dumps(output, ensure_ascii=False))

    except (KeyError, json.JSONDecodeError) as e:
        output = {
            "version": 1,
            "status": "error",
            "error_message": f"응답 파싱 오류: {e}",
            "lines": [{"text": f"❌ 응답 파싱 실패: {e}", "style": "error"}],
        }
        print(json.dumps(output, ensure_ascii=False))


if __name__ == "__main__":
    main()

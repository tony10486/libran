#!/usr/bin/env python3
"""Libran 시계 위젯 — WOP JSON을 stdout에 출력합니다."""
import json
from datetime import datetime

WEEKDAY_KR = ["월", "화", "수", "목", "금", "토", "일"]

now = datetime.now()

time_str = now.strftime("%H:%M:%S")
badge = now.strftime("%I:%M %p")
weekday = WEEKDAY_KR[now.weekday()]
date_str = now.strftime(f"%Y년 %m월 %d일 {weekday}요일")

output = {
    "version": 1,
    "status": "ok",
    "badge": badge,
    "lines": [
        {
            "text": time_str,
            "style": "bold",
            "align": "center",
            "icon": "🕐",
            "color": "FFAA00",
        },
        {
            "text": date_str,
            "style": "dim",
            "align": "center",
        },
    ],
}

print(json.dumps(output, ensure_ascii=False))

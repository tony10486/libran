# Libran 위젯 플러그인 제작 및 적용 가이드 (Widget Plugin Manual)

Libran은 사용자가 직접 사이드바와 팝업 패널에 표시할 미니 애플리케이션을 만들고 공유할 수 있도록 **선언적 TOML 매니페스트 + 스크립트/API 기반의 위젯 플러그인 기능**을 제공합니다.

Libran 코어 바이너리에는 어떤 위젯도 포함되어 있지 않습니다. 사용자가 원하는 위젯을 `~/.libran/widgets/` 디렉토리에 직접 설치하면, Libran이 시작 시 자동으로 탐색하여 로드합니다.

---

## 1. 개요 및 동작 방식

Libran의 위젯 플러그인은 컴파일된 코드가 아닌 **외부 스크립트 또는 HTTP API 호출**을 통해 동작합니다. 이를 통해 다음을 달성합니다:

- **코어 경량화**: Libran 바이너리에 위젯 코드가 포함되지 않습니다.
- **언어 독립성**: Python, Bash, Ruby, Node.js, Go 등 어떤 언어로든 위젯을 작성할 수 있습니다.
- **안전한 샌드박스**: 위젯은 권한이 제한된 환경에서 실행되며, 네트워크 접근·실행 시간·출력 크기가 제한됩니다.

### 위젯 유형

| 유형 | `type` | 동작 방식 | 적합한 용도 |
|---|---|---|---|
| **Script** | `"script"` | 외부 명령을 실행하고 stdout에서 WOP JSON을 수신 | 시계, 시스템 모니터, 일정, RSS 리더 |
| **API** | `"api"` | HTTP 요청을 보내고 응답을 선언적 필드 추출 | 날씨, 환율, arXiv 신간, GitHub 트렌드 |

### 디렉토리 구조

각 위젯은 `~/.libran/widgets/<위젯이름>/` 디렉토리에 하나의 단위로 설치됩니다:

```
~/.libran/widgets/
├── clock/
│   ├── widget.toml       # 매니페스트 (필수)
│   └── clock.py          # 스크립트 (script 타입)
├── weather/
│   ├── widget.toml
│   └── weather.py
├── arxiv-new/
│   ├── widget.toml       # api 타입 (스크립트 불필요)
│   └── (widget.toml만 있으면 됨)
└── schedule/
    ├── widget.toml
    ├── schedule.py
    └── schedule.json     # 위젯이 사용하는 데이터 파일 (선택)
```

### 위젯 표시 위치

위젯은 두 곳에 표시됩니다:

1. **사이드바 상단 위젯 바** (1줄): 모든 위젯의 `badge`를 한 줄로 표시
   - 예: `| 04:23 PM │ ⛅️ 23℃ │ 📋 2/5 │`
2. **위젯 패널** (`w` 키로 토글): 선택한 위젯의 상세 내용을 탭 형태로 표시

---

## 2. 매니페스트: `widget.toml`

모든 위젯의 루트에는 `widget.toml` 파일이 있어야 합니다. 이 파일이 위젯의 정체성, 실행 방법, 권한, 표시 방식을 정의합니다.

### 전체 구조

```toml
[widget]            # 위젯 메타데이터 (필수)
[script]            # Script 타입 설정 (type="script" 시 필수)
[api]               # API 타입 설정 (type="api" 시 필수)
[display]           # 표시 템플릿 (선택)
[permissions]       # 보안 권한 (선택, 권장)
```

### `[widget]` 섹션 (필수)

```toml
[widget]
name = "시계"                    # 표시 이름 (탭 바, 패널 타이틀)
id = "clock"                     # 고유 식별자 (영숫자, -, _ 만 허용)
version = "1.0.0"                # 버전 (기본값: "1.0.0")
description = "현재 시간을 표시합니다"  # 설명
type = "script"                  # "script" | "api"
refresh_interval = 1             # 자동 갱신 주기 (초). 0 = 수동만
enabled = true                   # 로드 여부 (기본값: true)
show_security_warning = false    # 최초 로드 시 보안 경고 표시 여부
```

> **`id` 규칙**: 영숫자, 하이픈(`-`), 언더스코어(`_`)만 허용. 빈 문자열 불가.
> **`refresh_interval`**: 0이면 사용자가 수동으로 새로고침(`r` 키)할 때만 갱신.

### `[script]` 섹션 (type="script" 시 필수)

```toml
[script]
command = "python3"              # 실행할 명령 (PATH에서 탐색됨)
args = ["clock.py"]              # 명령 인자 (위젯 디렉토리 기준 상대경로)

# 선택: 스크립트에 전달할 추가 환경변수
# PATH, HOME, LANG은 항상 포함되며 오버라이드 불가
[script.env]
TZ = "Asia/Seoul"
MY_API_KEY = "secret123"
```

스크립트는 **위젯 디렉토리를 작업 디렉토리로 하여** 실행됩니다:
- `cwd` = `~/.libran/widgets/<위젯이름>/`
- `stdin` = null (입력 불가)
- `stdout` = WOP JSON (파싱 대상)
- `stderr` = null (무시됨)
- 환경변수: `PATH`, `HOME`, `LANG` + `[script.env]`에 정의한 변수만

### `[api]` 섹션 (type="api" 시 필수)

```toml
[api]
url = "https://api.open-meteo.com/v1/forecast?latitude=37.57&longitude=126.98&current=temperature_2m"
method = "GET"                   # HTTP 메서드 (기본값: "GET")
response_format = "json"         # "json" | "xml" | "text"

# 선택: 요청 헤더
[api.headers]
Authorization = "Bearer token123"
Accept = "application/json"

# 선택: 응답에서 필드 추출
[api.extract]
items = "/data/items"            # JSON 포인터 (배열 경로) 또는 XML 태그명
value = "/current/temperature"   # 단일 값 추출 (items 없이 사용)

[[api.extract.fields]]           # 각 항목에서 추출할 필드
name = "title"                   # 필드명 (템플릿에서 {title}로 참조)
path = "/title"                  # JSON 포인터 또는 XML 태그명
default = "제목 없음"             # 값이 없을 때 기본값 (선택)

[[api.extract.fields]]
name = "url"
path = "/link"
```

**JSON 포인터 문법** (RFC 6901):
- `/` — 루트
- `/foo` — `foo` 키
- `/foo/0` — `foo` 배열의 첫 번째 요소
- `/a/b` — 중첩 경로 `a.b`

**추출 결과**는 `[display]`의 템플릿에 전달됩니다.

### `[display]` 섹션 (선택)

```toml
[display]
item_template = "{title} — {date}"   # 각 항목 렌더 템플릿 ({field} 플레이스홀더)
detail_template = "{description}"    # 항목 상세 줄 (선택)
max_items = 20                       # 최대 표시 항목 수 (기본값: 20)
empty_message = "데이터가 없습니다"    # 항목이 없을 때 메시지 (기본값: "데이터 없음")
date_format = "%Y-%m-%d"             # 날짜 필드 포맷 (chrono 포맷, 선택)
```

> `[display]`는 API 타입에서 주로 사용됩니다. Script 타입은 스크립트가 직접 WOP JSON을 생성하므로 `[display]`가 필요하지 않습니다 (사용해도 무방).

### `[permissions]` 섹션 (선택, 권장)

```toml
[permissions]
network = ["api.open-meteo.com", "export.arxiv.org"]  # 허용된 도메인
max_execution_time = 10          # 스크립트 타임아웃 (초, 기본값: 10)
max_output_bytes = 65536         # stdout 최대 바이트 (기본값: 65536 = 64KB)
```

> **`network`**: Script 타입에서 HTTP 요청이 필요한 경우, 샌드박스가 이 도메인 목록 + 전역 허용 목록(`config.toml`의 `[widgets] allowed_domains`)의 합집합으로 접근을 제한합니다. API 타입은 `url`의 도메인이 자동으로 허용 목록에 추가됩니다.

---

## 3. Widget Output Protocol (WOP)

모든 Script 위젯은 stdout에 **WOP JSON**을 출력해야 합니다. API 위젯은 Libran이 응답을 WOP 형식으로 변환합니다. WOP는 위젯의 렌더링 데이터를 구조적으로 표현하는 JSON 스키마입니다.

### WOP JSON 스키마

```json
{
  "version": 1,
  "status": "ok",
  "title": "현재 시간",
  "badge": "04:23 PM",
  "lines": [
    {
      "text": "14:23:45",
      "style": "bold",
      "color": "FF8800",
      "align": "center",
      "icon": "🕐"
    }
  ],
  "sections": [
    {
      "header": "오늘의 일정",
      "lines": [
        { "text": "팀 미팅 10:00", "style": "normal" },
        { "text": "리뷰 14:00", "style": "dim" }
      ]
    }
  ],
  "error_message": null,
  "next_refresh": 60,
  "actions": [
    {
      "key": "r",
      "label": "새로고침",
      "action": "refresh",
      "payload": null
    },
    {
      "key": "o",
      "label": "링크 열기",
      "action": "open_url",
      "payload": "https://example.com"
    }
  ]
}
```

### 필드 상세

| 필드 | 타입 | 필수 | 설명 |
|---|---|---|---|
| `version` | `number` | 예 | WOP 프로토콜 버전. 현재 `1`만 지원 |
| `status` | `"ok"` \| `"error"` \| `"loading"` | 예 | 위젯 상태. 패널 렌더링 분기에 사용 |
| `title` | `string` | 아니오 | 패널 타이틀 오버라이드. 없으면 manifest의 `name` 사용 |
| `badge` | `string` | 아니오 | 사이드바 위젯 바에 표시할 컴팩트 요약. 없으면 바에 표시 안 함 |
| `lines` | `WidgetLine[]` | 아니오 | 본문 텍스트 라인. `sections`와 함께 또는 단독 사용 |
| `sections` | `WidgetSection[]` | 아니오 | 논리 섹션 그룹. 헤더 + 라인 목록 |
| `error_message` | `string` | 아니오 | `status: "error"`일 때 상세 메시지 |
| `next_refresh` | `number` | 아니오 | 다음 갱신까지 초. manifest의 `refresh_interval` 오버라이드 |
| `actions` | `WidgetAction[]` | 아니오 | 사용자가 위젯 패널에서 실행할 수 있는 커스텀 액션 |

### `WidgetLine` 스키마

| 필드 | 타입 | 기본값 | 설명 |
|---|---|---|---|
| `text` | `string` | `""` | 표시할 텍스트 |
| `style` | `string` | `"normal"` | `"normal"` \| `"bold"` \| `"dim"` \| `"italic"` \| `"highlight"` \| `"error"` \| `"success"` \| `"warning"` |
| `color` | `string` | `null` | 16진수 색상 코드 (`"RRGGBB"` 또는 `"#RRGGBB"`). 테마 색상을 오버라이드 |
| `align` | `string` | `"left"` | `"left"` \| `"center"` \| `"right"` |
| `icon` | `string` | `null` | 텍스트 앞에 붙이는 유니코드 아이콘 (예: `"🕐"`, `"⛅️"`) |

### `WidgetSection` 스키마

| 필드 | 타입 | 설명 |
|---|---|---|
| `header` | `string` | 섹션 헤더 텍스트 |
| `lines` | `WidgetLine[]` | 섹션 본문 라인 |

### `WidgetAction` 스키마

위젯 패널 하단 힌트 바에 표시되는 커스텀 액션입니다.

| 필드 | 타입 | 설명 |
|---|---|---|
| `key` | `string` (1字符) | 단축키. 영숫자만 허용 |
| `label` | `string` | 힌트 바에 표시할 라벨 |
| `action` | `string` | `"refresh"` \| `"open_url"` \| `"custom"` |
| `payload` | `string` | `action="open_url"` 시 URL. `action="custom"` 시 임의 문자열 |

**액션 동작**:
- `"refresh"`: 위젯을 즉시 재실행/재요청
- `"open_url"`: 시스템 기본 브라우저로 `payload` URL 열기
- `"custom"`: Script 위젯의 경우 `action_command`가 정의되어 있으면 실행. 없으면 무시

### 최소 WOP 예시

```json
{"version": 1, "status": "ok", "lines": [{"text": "Hello, World!"}]}
```

### badge만 있는 예시 (위젯 바 전용)

```json
{"version": 1, "status": "ok", "badge": "04:23 PM"}
```

---

## 4. 스크립트 위젯 제작

### 4.1 Python 예시: 시계 위젯

**`~/.libran/widgets/clock/widget.toml`**
```toml
[widget]
name = "시계"
id = "clock"
version = "1.0.0"
description = "현재 시간 표시"
type = "script"
refresh_interval = 1
enabled = true

[script]
command = "python3"
args = ["clock.py"]

[script.env]
TZ = "Asia/Seoul"

[permissions]
network = []
max_execution_time = 3
max_output_bytes = 4096
```

**`~/.libran/widgets/clock/clock.py`**
```python
#!/usr/bin/env python3
import json
from datetime import datetime
import os

tz = os.environ.get("TZ", "UTC")
now = datetime.now()

time_str = now.strftime("%H:%M:%S")
badge = now.strftime("%I:%M %p")  # 04:23 PM
weekday_kr = ["월", "화", "수", "목", "금", "토", "일"][now.weekday()]
date_str = now.strftime(f"%Y년 %m월 %d일 {weekday_kr}요일")

output = {
    "version": 1,
    "status": "ok",
    "badge": badge,
    "lines": [
        {"text": time_str, "style": "bold", "align": "center", "icon": "🕐", "color": "FFAA00"},
        {"text": date_str, "style": "dim", "align": "center"},
    ],
}
print(json.dumps(output, ensure_ascii=False))
```

### 4.2 Bash 예시: 시스템 모니터

**`~/.libran/widgets/sysmon/widget.toml`**
```toml
[widget]
name = "시스템 모니터"
id = "sysmon"
version = "1.0.0"
description = "CPU/메모리 사용률"
type = "script"
refresh_interval = 5
enabled = true

[script]
command = "bash"
args = ["monitor.sh"]

[permissions]
network = []
max_execution_time = 5
max_output_bytes = 8192
```

**`~/.libran/widgets/sysmon/monitor.sh`**
```bash
#!/usr/bin/env bash
set -euo pipefail

# CPU 사용률 (macOS: top, Linux: /proc/stat)
cpu=$(top -l 1 -n 0 2>/dev/null | grep "CPU usage" | awk '{print $3}' || echo "N/A")
mem_used=$(vm_stat | awk '/Pages active/ {print $3}' | tr -d '.' 2>/dev/null || echo "0")
mem_total=$(sysctl -n hw.memsize 2>/dev/null || echo "0")

# 메모리 백분율 계산 (단순화)
mem_pct="N/A"
if [[ "$mem_total" != "0" && -n "$mem_used" ]]; then
    mem_pct=$(echo "scale=0; $mem_used * 4096 * 100 / $mem_total" | bc 2>/dev/null || echo "N/A")
fi

# WOP JSON 출력
cat <<EOF
{
  "version": 1,
  "status": "ok",
  "badge": "CPU ${cpu}%",
  "lines": [
    {"text": "CPU 사용률: ${cpu}", "style": "bold", "icon": "🖥"},
    {"text": "메모리 활성: ${mem_pct}%", "style": "normal", "icon": "💾"}
  ]
}
EOF
```

### 4.3 액션 기반 위젯: 일정 관리 (상호작용)

일정 추가/토글/삭제 같은 상호작용이 필요한 위젯은 `action_command`를 사용합니다.

**`~/.libran/widgets/schedule/widget.toml`**
```toml
[widget]
name = "일정"
id = "schedule"
version = "1.0.0"
description = "간단한 일정 관리"
type = "script"
refresh_interval = 60
enabled = true

[script]
command = "python3"
args = ["schedule.py"]

[permissions]
network = []
max_execution_time = 5
max_output_bytes = 16384
```

**`~/.libran/widgets/schedule/schedule.py`**
```python
#!/usr/bin/env python3
import json
import os
from datetime import datetime

WIDGET_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_FILE = os.path.join(WIDGET_DIR, "schedule.json")

def load_data():
    if os.path.exists(DATA_FILE):
        with open(DATA_FILE) as f:
            return json.load(f)
    return {"items": [], "next_id": 0}

def save_data(data):
    with open(DATA_FILE, "w") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

def render(data):
    items = data["items"]
    today = datetime.now().strftime("%Y-%m-%d")
    done = sum(1 for i in items if i.get("done"))
    total = len(items)

    lines = []
    for item in items:
        mark = "✅" if item.get("done") else "⬜"
        due = f" ({item['due_date']})" if item.get("due_date") else ""
        lines.append({"text": f"{mark} {item['title']}{due}", "style": "dim" if item.get("done") else "normal"})

    badge = f"📋 {done}/{total}" if total > 0 else None

    output = {
        "version": 1,
        "status": "ok",
        "badge": badge,
        "lines": lines or [{"text": "일정이 없습니다", "style": "dim"}],
        "actions": [
            {"key": "a", "label": "추가", "action": "custom", "payload": "add"},
            {"key": " ", "label": "토글", "action": "custom", "payload": "toggle"},
            {"key": "d", "label": "삭제", "action": "custom", "payload": "delete"},
        ],
    }
    print(json.dumps(output, ensure_ascii=False))

if __name__ == "__main__":
    render(load_data())
```

> **참고**: `action`이 `"custom"`인 액션은 향후 `action_command` 기능이 구현되면 스크립트의 `--action` 모드로 전달됩니다. 1차 구현에서는 custom 액션이 무시되며, 위젯은 표시 전용으로 동작합니다.

---

## 5. API 위젯 제작

API 위젯은 스크립트 없이 `widget.toml`만으로 동작합니다. Libran이 HTTP 요청을 수행하고 응답을 파싱하여 WOP로 변환합니다.

### 5.1 날씨 위젯 (Open-Meteo)

**`~/.libran/widgets/weather/widget.toml`**
```toml
[widget]
name = "날씨"
id = "weather"
version = "1.0.0"
description = "현재 날씨 (Open-Meteo)"
type = "api"
refresh_interval = 600
enabled = true

[api]
url = "https://api.open-meteo.com/v1/forecast?latitude=37.5665&longitude=126.9780&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weathercode,is_day&timezone=auto"
method = "GET"
response_format = "json"

[api.extract]
value = "/current/temperature_2m"

[[api.extract.fields]]
name = "temperature"
path = "/current/temperature_2m"

[[api.extract.fields]]
name = "apparent_temp"
path = "/current/apparent_temperature"

[[api.extract.fields]]
name = "humidity"
path = "/current/relative_humidity_2m"

[[api.extract.fields]]
name = "wind_speed"
path = "/current/wind_speed_10m"

[[api.extract.fields]]
name = "weather_code"
path = "/current/weathercode"

[display]
item_template = "🌡 온도: {temperature}°C (체감 {apparent_temp}°C)"
max_items = 5
empty_message = "날씨 정보를 불러올 수 없습니다"

[permissions]
network = ["api.open-meteo.com"]
```

### 5.2 arXiv 신간 위젯

**`~/.libran/widgets/arxiv/widget.toml`**
```toml
[widget]
name = "arXiv 신간"
id = "arxiv"
version = "1.0.0"
description = "cs.AI 최신 논문 5편"
type = "api"
refresh_interval = 3600
enabled = true

[api]
url = "http://export.arxiv.org/api/query?search_query=cat:cs.AI&sortBy=submittedDate&sortOrder=descending&max_results=5"
method = "GET"
response_format = "xml"

[api.extract]
items = "//entry"

[[api.extract.fields]]
name = "title"
path = "/title"

[[api.extract.fields]]
name = "summary"
path = "/summary"

[[api.extract.fields]]
name = "link"
path = "/id"

[display]
item_template = "📄 {title}"
detail_template = "{summary}"
max_items = 5
empty_message = "논문 없음"

[permissions]
network = ["export.arxiv.org"]
```

---

## 6. 보안: 샌드박스 모델

Libran은 위젯이 시스템에 미치는 영향을 최소화하기 위해 다중 계층의 샌드박스를 적용합니다.

### 6.1 네트워크 제한

- **HTTPS 강제**: 모든 HTTP 요청은 HTTPS여야 합니다 (API 타입 + Script 타입의 HTTP 호출).
- **도메인 허용 목록**: `[permissions] network`에 명시된 도메인 + 전역 설정(`config.toml`의 `[widgets] allowed_domains`)의 합집합만 접근 가능.
- **속도 제한**: 토큰 버킷 알고리즘으로 도메인별 요청 빈도를 제한.

### 6.2 스크립트 실행 제한

| 제한 | 기본값 | 설정 |
|---|---|---|
| 실행 타임아웃 | 10초 | `[permissions] max_execution_time` |
| stdout 최대 크기 | 64KB | `[permissions] max_output_bytes` |
| 작업 디렉토리 | 위젯 디렉토리로 격리 | 자동 |
| 환경변수 | `PATH`, `HOME`, `LANG` + `[script.env]`만 | 자동 (`env_clear`) |
| stdin | null (입력 차단) | 자동 |
| stderr | null (무시) | 자동 |

### 6.3 보안 경고

`show_security_warning = true`로 설정하면, 위젯이 처음 로드될 때 사용자에게 보안 경고를 표시합니다. Script 타입 위젯은 `true`로 설정하는 것을 권장합니다.

---

## 7. 전역 위젯 설정 (`config.toml`)

Libran의 메인 설정 파일(`~/.libran/config.toml`)에 `[widgets]` 섹션을 통해 전역 위젯 동작을 제어할 수 있습니다.

```toml
[widgets]
enabled = true                    # 위젯 시스템 전체 활성화 (기본값: true)
tick_interval_secs = 1            # 위젯 틱 간격 (초, 기본값: 1)
allowed_domains = []              # 전역으로 허용할 추가 도메인 (기본값: 빈 배열)
```

- **`tick_interval_secs`**: Libran이 위젯 새로고침 필요 여부를 확인하는 주기. 각 위젯의 `refresh_interval`과 독립적입니다. 예: `tick_interval_secs = 1`이고 위젯의 `refresh_interval = 60`이면, Libran은 매초 확인하지만 위젯은 60초마다만 재실행합니다.
- **`allowed_domains`**: 모든 위젯이 공통으로 접근할 수 있는 도메인. 예: `["api.open-meteo.com"]`

---

## 8. 위젯 배포 및 설치

### 수동 설치

```bash
# 위젯 디렉토리 생성
mkdir -p ~/.libran/widgets/clock

# 파일 복사
cp widget.toml clock.py ~/.libran/widgets/clock/

# 스크립트 실행 권한 (필요한 경우)
chmod +x ~/.libran/widgets/clock/clock.py
```

Libran 재시작 시 자동으로 `~/.libran/widgets/`를 스캔하여 `widget.toml`이 있는 디렉토리를 위젯으로 로드합니다. `enabled = false`인 위젯은 로드되지 않습니다.

### 위젯 디버깅

위젯 스크립트가 제대로 작동하는지 터미널에서 직접 테스트:

```bash
cd ~/.libran/widgets/clock
python3 clock.py
# WOP JSON이 stdout에 출력되어야 함
```

WOP JSON이 유효한지 확인:

```bash
python3 clock.py | python3 -m json.tool
```

### 일반적인 문제

| 즐상 | 원인 | 해결 |
|---|---|---|
| 위젯이 로드되지 않음 | `widget.toml` 파싱 오류 | 터미널에서 `toml-cli validate widget.toml` 실행 |
| 위젯이 "로딩 중"에서 멈춤 | 스크립트 타임아웃 | `max_execution_time` 증가, 스크립트 성능 점검 |
| "스크립트 오류" 표시 | 스크립트가 WOP JSON을 출력하지 않음 | 스크립트가 `json.dumps()` 후 `print`하는지 확인 |
| "WOP JSON 파싱 실패" | 출력이 JSON이 아님 | 스크립트가 stderr가 아닌 stdout에 JSON을 출력하는지 확인 |
| 네트워크 오류 | 도메인이 허용 목록에 없음 | `[permissions] network`에 도메인 추가 |
| 위젯 바에 표시되지 않음 | `badge` 필드가 없음 | WOP JSON에 `"badge": "..."` 추가 |

---

## 9. WOP JSON 빠른 참조

### status: "ok"
```json
{
  "version": 1,
  "status": "ok",
  "badge": "04:23 PM",
  "lines": [
    {"text": "14:23:45", "style": "bold", "color": "FFAA00", "align": "center", "icon": "🕐"}
  ]
}
```

### status: "loading"
```json
{
  "version": 1,
  "status": "loading",
  "lines": [{"text": "⏳ 로딩 중...", "style": "dim"}]
}
```

### status: "error"
```json
{
  "version": 1,
  "status": "error",
  "error_message": "API 키가 만료되었습니다",
  "lines": [{"text": "❌ API 키가 만료되었습니다", "style": "error"}]
}
```

### sections 사용
```json
{
  "version": 1,
  "status": "ok",
  "sections": [
    {
      "header": "오늘",
      "lines": [
        {"text": "팀 미팅 10:00", "style": "bold"},
        {"text": "리뷰 14:00", "style": "normal"}
      ]
    },
    {
      "header": "내일",
      "lines": [
        {"text": "데모 09:00", "style": "warning"}
      ]
    }
  ]
}
```

### actions 포함
```json
{
  "version": 1,
  "status": "ok",
  "badge": "⛅️ 23℃",
  "lines": [{"text": "서울 23°C 맑음", "style": "bold"}],
  "actions": [
    {"key": "r", "label": "새로고침", "action": "refresh"},
    {"key": "o", "label": "상세", "action": "open_url", "payload": "https://weather.com/..."}
  ]
}
```

---

## 10. 위젯 제작 체크리스트

- [ ] `widget.toml`에 `[widget]` 섹션 작성 (`id`는 고유값)
- [ ] `type`에 맞는 섹션 작성 (`[script]` 또는 `[api]`)
- [ ] `[permissions]`에 필요한 도메인과 제한 값 설정
- [ ] Script 타입: 스크립트가 stdout에 WOP JSON을 출력하는지 확인
- [ ] `badge` 필드 추가 (사이드바 위젯 바에 표시하려면)
- [ ] `refresh_interval` 설정 (너무 짧으면 시스템 부하, 너무 길면 정보 지연)
- [ ] 에러 상황에서도 WOP JSON을 출력하도록 처리 (`status: "error"`)
- [ ] 스크립트에 실행 권한 부여 (`chmod +x`)
- [ ] 터미널에서 직접 실행하여 WOP JSON 검증
- [ ] `show_security_warning` 설정 (외부 네트워크 접근 시 `true` 권장)

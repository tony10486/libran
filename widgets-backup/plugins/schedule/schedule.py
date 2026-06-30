#!/usr/bin/env python3
"""Libran 일정 위젯 — schedule.json에서 일정을 읽어 WOP JSON으로 출력합니다.

데이터 파일: 위젯 디렉토리 내 schedule.json
형식:
{
  "items": [
    {"id": 0, "title": "...", "due_date": "YYYY-MM-DD", "done": false, "created_at": "..."}
  ],
  "next_id": 1
}

참고: 이 위젯은 표시 전용입니다. 추가/토글/삭제는 schedule.py를 직접 실행하여
할 수 있습니다:
  python3 schedule.py --add "제목" "2024-12-31"
  python3 schedule.py --toggle 0
  python3 schedule.py --delete 0
"""
import json
import os
import sys
from datetime import datetime

WIDGET_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_FILE = os.path.join(WIDGET_DIR, "schedule.json")


def load_data():
    if os.path.exists(DATA_FILE):
        try:
            with open(DATA_FILE, "r", encoding="utf-8") as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError):
            pass
    return {"items": [], "next_id": 0}


def save_data(data):
    with open(DATA_FILE, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)


def cmd_add(args):
    """schedule.py --add "제목" ["YYYY-MM-DD"]"""
    if len(args) < 1:
        print("사용법: --add \"제목\" [YYYY-MM-DD]", file=sys.stderr)
        sys.exit(1)
    title = args[0]
    due = args[1] if len(args) > 1 else None
    data = load_data()
    item = {
        "id": data["next_id"],
        "title": title,
        "due_date": due,
        "done": False,
        "created_at": datetime.now().strftime("%Y-%m-%d"),
    }
    data["items"].append(item)
    data["next_id"] += 1
    save_data(data)
    print(f"추가됨: {title}")


def cmd_toggle(args):
    """schedule.py --toggle <id>"""
    if len(args) < 1:
        print("사용법: --toggle <id>", file=sys.stderr)
        sys.exit(1)
    item_id = int(args[0])
    data = load_data()
    for item in data["items"]:
        if item["id"] == item_id:
            item["done"] = not item["done"]
            save_data(data)
            print(f"토글됨: {item['title']}")
            return
    print(f"ID {item_id}를 찾을 수 없습니다", file=sys.stderr)
    sys.exit(1)


def cmd_delete(args):
    """schedule.py --delete <id>"""
    if len(args) < 1:
        print("사용법: --delete <id>", file=sys.stderr)
        sys.exit(1)
    item_id = int(args[0])
    data = load_data()
    before = len(data["items"])
    data["items"] = [i for i in data["items"] if i["id"] != item_id]
    if len(data["items"]) < before:
        save_data(data)
        print(f"삭제됨: ID {item_id}")
    else:
        print(f"ID {item_id}를 찾을 수 없습니다", file=sys.stderr)
        sys.exit(1)


def render():
    """WOP JSON을 stdout에 출력 (위젯 렌더링용)"""
    data = load_data()
    items = data["items"]
    today = datetime.now().strftime("%Y-%m-%d")

    done_count = sum(1 for i in items if i.get("done"))
    total = len(items)

    # 정렬: 미완료 먼저, 그 다음 마감일 순
    def sort_key(item):
        done = 1 if item.get("done") else 0
        due = item.get("due_date") or "9999-12-31"
        return (done, due)

    sorted_items = sorted(items, key=sort_key)

    lines = []
    for item in sorted_items:
        mark = "✅" if item.get("done") else "⬜"
        title = item["title"]
        due = item.get("due_date")
        style = "dim" if item.get("done") else "normal"

        text = f"{mark} {title}"
        if due:
            # 오늘 마감이면 강조
            if due == today and not item.get("done"):
                style = "warning"
                text += f" (오늘 마감!)"
            else:
                text += f" ({due})"
        lines.append({"text": text, "style": style})

    if not lines:
        lines = [{"text": "일정이 없습니다", "style": "dim"}]

    badge = f"📋 {done_count}/{total}" if total > 0 else None

    output = {
        "version": 1,
        "status": "ok",
        "badge": badge,
        "lines": lines,
    }
    print(json.dumps(output, ensure_ascii=False))


if __name__ == "__main__":
    args = sys.argv[1:]
    if args and args[0] == "--add":
        cmd_add(args[1:])
    elif args and args[0] == "--toggle":
        cmd_toggle(args[1:])
    elif args and args[0] == "--delete":
        cmd_delete(args[1:])
    else:
        render()

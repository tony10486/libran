"""Print every key-value pair in opencode.json with no dedup issues."""
import json, os, sys

path = os.path.expanduser("~/.config/opencode/opencode.json")
with open(path) as f:
    data = json.load(f)

# Flatten and print everything
def emit(k, v, depth=0):
    prefix = "  " * depth
    if isinstance(v, dict):
        for sk, sv in v.items():
            emit(f"{k}.{sk}", sv, depth+1)
    elif isinstance(v, list):
        for i, item in enumerate(v):
            emit(f"{k}[{i}]", item, depth+1)
    else:
        print(f"{prefix}{k}: {v}")

for k, v in data.items():
    emit(k, v)

sys.stdout.flush()
print("\n===== DONE =====")

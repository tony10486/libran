import json, os, uuid, random

with open(os.path.expanduser("~/.config/opencode/opencode.json")) as f:
    data = json.load(f)

uid = uuid.uuid4().hex[:6]
rand = random.randint(0, 99999)
print(f"UID={uid} R={rand}")

print(f"TOP_KEYS={list(data.keys())}")

g = data["provider"]["google"]
print(f"GOOGLE_KEYS={list(g.keys())}")
print(f"GOOGLE_NPM={g.get('npm','MISSING')}")
print(f"GOOGLE_MODELS_COUNT={len(g.get('models',{}))}")

for pn in data["provider"]:
    p = data["provider"][pn]
    print(f"PROV[{pn}] npm={p.get('npm','?')} name={p.get('name','?')} models={len(p.get('models',{}))}")

print(f"PLUGINS={json.dumps(data.get('plugin',[]))}")
print(f"TEAM_MODE={json.dumps(data.get('team_mode',{}))}")

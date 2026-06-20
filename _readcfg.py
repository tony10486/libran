import json, os

with open(os.path.expanduser("~/.config/opencode/opencode.json")) as f:
    data = json.load(f)

# Check for misplaced keys (keys that should be inside provider but are at top level)
print("=== TOP LEVEL ===")
for k in data:
    print(f"  [{k}]")

print()
print("=== TEAM_MODE ===")
print(json.dumps(data.get("team_mode", {}), indent=2))

print()
print("=== PLUGINS ===")
for p in data.get("plugin", []):
    if isinstance(p, dict):
        print(f"  {json.dumps(p, indent=2)}")
    else:
        print(f"  {p}")

print()
print("=== PROVIDERS ===")
for pname, pcfg in data.get("provider", {}).items():
    print(f"Provider [{pname}]:")
    print(f"  npm: {pcfg.get('npm', 'N/A')}")
    print(f"  name: {pcfg.get('name', 'N/A')}")
    models = pcfg.get("models", {})
    for mname, mcfg in models.items():
        print(f"  Model [{mname}]:")
        print(f"    {json.dumps(mcfg, ensure_ascii=False)}")

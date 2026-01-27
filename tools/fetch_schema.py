#!/usr/bin/env python3
import argparse
import json
import os
import sys
import urllib.request
from pathlib import Path

def read_from_url(url: str) -> str:
    with urllib.request.urlopen(url) as resp:
        return resp.read().decode("utf-8")

def read_from_repo(repo_path: Path) -> str:
    swagger_path = repo_path / "swagger" / "swagger.json"
    if not swagger_path.exists():
        raise FileNotFoundError(f"swagger.json not found at {swagger_path}")
    return swagger_path.read_text(encoding="utf-8")

def main() -> int:
    parser = argparse.ArgumentParser(description="Fetch Chatwoot OpenAPI schema.")
    parser.add_argument("--out", default="schemas/swagger.json")
    parser.add_argument("--url", default=os.getenv("CHATWOOT_SWAGGER_URL"))
    parser.add_argument(
        "--repo",
        default=os.getenv("CHATWOOT_SOURCE_REPO", "/Users/radjathaher/github.com/forks/chatwoot"),
    )
    args = parser.parse_args()

    if args.url:
        raw = read_from_url(args.url)
    else:
        raw = read_from_repo(Path(args.repo))

    data = json.loads(raw)
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(data, indent=2, sort_keys=True), encoding="utf-8")
    print(out_path)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

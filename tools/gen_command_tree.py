#!/usr/bin/env python3
import argparse
import json
import re
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

CAMEL_RE = re.compile(r"([a-z0-9])([A-Z])")
NON_ALNUM = re.compile(r"[^a-zA-Z0-9]+")


def to_kebab(value: str) -> str:
    value = CAMEL_RE.sub(r"\1-\2", value)
    value = NON_ALNUM.sub("-", value.strip())
    value = value.strip("-")
    return value.lower()


def resolve_ref(obj: Any, spec: Dict[str, Any]) -> Any:
    if isinstance(obj, dict) and "$ref" in obj:
        ref = obj["$ref"]
        if not ref.startswith("#/"):
            return obj
        parts = ref.lstrip("#/").split("/")
        cur: Any = spec
        for part in parts:
            cur = cur[part]
        return resolve_ref(cur, spec)
    return obj


def merge_all_of(schema: Dict[str, Any]) -> Dict[str, Any]:
    if "allOf" not in schema:
        return schema
    merged: Dict[str, Any] = {"type": "object", "properties": {}, "required": []}
    for part in schema.get("allOf", []):
        resolved = merge_all_of(resolve_ref(part, spec))
        props = resolved.get("properties", {})
        merged["properties"].update(props)
        merged["required"].extend(resolved.get("required", []))
    return merged


def schema_type(schema: Dict[str, Any]) -> Optional[str]:
    if not schema:
        return None
    if "type" in schema:
        return schema.get("type")
    if "$ref" in schema:
        return schema["$ref"].split("/")[-1]
    if "allOf" in schema:
        return "object"
    if "oneOf" in schema or "anyOf" in schema:
        return "object"
    return None


def extract_properties(schema: Dict[str, Any]) -> Tuple[List[Dict[str, Any]], List[str]]:
    schema = merge_all_of(resolve_ref(schema, spec))
    if schema.get("type") != "object":
        return [], []
    props = schema.get("properties", {}) or {}
    required = schema.get("required", []) or []
    fields = []
    for name, prop in props.items():
        prop = resolve_ref(prop, spec)
        fields.append(
            {
                "name": name,
                "flag": f"input-{to_kebab(name)}",
                "required": name in required,
                "schema_type": schema_type(prop),
                "description": prop.get("description"),
            }
        )
    fields.sort(key=lambda f: f["name"])
    return fields, sorted(set(required))


def build_params(path_params: List[Any], op_params: List[Any]) -> List[Dict[str, Any]]:
    merged: Dict[Tuple[str, str], Dict[str, Any]] = {}
    for param in path_params + op_params:
        resolved = resolve_ref(param, spec)
        location = resolved.get("in")
        name = resolved.get("name")
        if not location or not name:
            continue
        if location == "header":
            continue
        key = (name, location)
        merged[key] = resolved
    params = []
    for (name, location), param in merged.items():
        schema = resolve_ref(param.get("schema", {}), spec)
        p_type = schema_type(schema)
        params.append(
            {
                "name": name,
                "flag": to_kebab(name),
                "location": location,
                "required": bool(param.get("required") or location == "path"),
                "schema_type": p_type,
                "description": param.get("description"),
                "is_array": p_type == "array",
            }
        )
    params.sort(key=lambda p: (p["location"], p["name"]))
    return params


def build_request_body(op: Dict[str, Any]) -> Dict[str, Any] | None:
    request_body = op.get("requestBody")
    if not request_body:
        return None
    request_body = resolve_ref(request_body, spec)
    content = request_body.get("content", {}) or {}
    content_types = sorted(content.keys())
    schema = None
    if "application/json" in content:
        schema = content["application/json"].get("schema")
    elif "application/x-www-form-urlencoded" in content:
        schema = content["application/x-www-form-urlencoded"].get("schema")
    elif content_types:
        schema = content[content_types[0]].get("schema")
    fields, required_fields = extract_properties(schema or {})
    return {
        "required": bool(request_body.get("required")),
        "content_types": content_types,
        "input_fields": fields,
        "required_fields": required_fields,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate Chatwoot CLI command tree from OpenAPI.")
    parser.add_argument("--schema", default="schemas/swagger.json")
    parser.add_argument("--out", default="schemas/command_tree.json")
    args = parser.parse_args()

    schema_path = Path(args.schema)
    if not schema_path.exists():
        raise FileNotFoundError(schema_path)

    global spec
    spec = json.loads(schema_path.read_text(encoding="utf-8"))

    resources: Dict[str, Dict[str, Any]] = {}
    for path, methods in (spec.get("paths") or {}).items():
        path_params = methods.get("parameters", []) if isinstance(methods, dict) else []
        for method, op in (methods or {}).items():
            if method == "parameters":
                continue
            op_id = op.get("operationId")
            if not op_id:
                continue
            tags = op.get("tags") or ["default"]
            params = build_params(path_params, op.get("parameters", []) or [])
            request_body = build_request_body(op)
            security = []
            for entry in op.get("security", []) or []:
                security.extend(entry.keys())
            for tag in tags:
                res_name = to_kebab(tag)
                resource = resources.setdefault(res_name, {"name": res_name, "ops": []})
                op_name = to_kebab(op_id)
                existing = {o["name"] for o in resource["ops"]}
                if op_name in existing:
                    op_name = f"{op_name}-{method}"
                resource["ops"].append(
                    {
                        "name": op_name,
                        "method": method.lower(),
                        "path": path,
                        "summary": op.get("summary"),
                        "description": op.get("description"),
                        "params": params,
                        "request_body": request_body,
                        "security": sorted(set(security)),
                    }
                )

    for res in resources.values():
        res["ops"].sort(key=lambda op: op["name"])

    tree = {
        "version": 1,
        "base_url": (spec.get("servers") or [{"url": ""}])[0].get("url", ""),
        "resources": [resources[key] for key in sorted(resources.keys())],
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(tree, indent=2, sort_keys=True), encoding="utf-8")
    print(out_path)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

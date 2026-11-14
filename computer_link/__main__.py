"""JSON-line local process interface, intentionally not an MCP transport."""
import argparse
import json
import sys
from . import diagnostics_registry


def respond(registry, request):
    if not isinstance(request, dict) or set(request) - {"id", "method", "params"}:
        return {"error": "invalid request"}
    identity = request.get("id")
    if not (identity is None or type(identity) in (str, int)):
        return {"error": "invalid id"}
    try:
        method = request.get("method")
        if not isinstance(method, str):
            raise ValueError("invalid method")
        if method == "capabilities.list":
            if request.get("params") not in (None, {}):
                raise ValueError("unexpected parameters")
            result = registry.describe()
        else:
            result = registry.call(method, request.get("params"))
        return {"id": identity, "result": result}
    except (ValueError, PermissionError) as error:
        return {"id": identity, "error": str(error)}
    except OSError:
        return {"id": identity, "error": "operating system unavailable"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", help="Optional user-selected directory for disk capacity only")
    args = parser.parse_args()
    try:
        registry = diagnostics_registry(args.workspace)
    except (ValueError, OSError):
        parser.error("workspace is unavailable or not a directory")
    for line in sys.stdin:
        try:
            request = json.loads(line)
            response = respond(registry, request)
        except (ValueError, RecursionError):
            response = {"error": "invalid JSON"}
        print(json.dumps(response, ensure_ascii=True), flush=True)


if __name__ == "__main__":
    main()

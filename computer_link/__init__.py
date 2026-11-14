"""Computer Link: independently usable, local-only diagnostics foundation."""
from dataclasses import dataclass
import platform
import shutil
from pathlib import Path


@dataclass(frozen=True)
class Capability:
    name: str
    effect: str
    handler: object


class Registry:
    def __init__(self):
        self._capabilities = {}

    def register(self, capability):
        if capability.name in self._capabilities:
            raise ValueError("duplicate capability")
        if capability.effect not in {"read", "write"}:
            raise ValueError("invalid effect")
        self._capabilities[capability.name] = capability

    def remove(self, name):
        self._capabilities.pop(name, None)

    def describe(self):
        return [{"name": c.name, "effect": c.effect}
                for c in sorted(self._capabilities.values(), key=lambda c: c.name)]

    def call(self, name, params=None, *, read_only=True):
        c = self._capabilities.get(name)
        if c is None:
            raise ValueError("unsupported capability")
        if read_only and c.effect != "read":
            raise PermissionError("read-only session")
        if params not in (None, {}):
            raise ValueError("unexpected parameters")
        return c.handler()


def diagnostics_registry(workspace=None):
    registry = Registry()
    registry.register(Capability("system.summary", "read", lambda: {
        "os": platform.system(), "architecture": platform.machine(),
        "python": platform.python_version(), "network_listener": False,
        "desktop_control": False, "mcp_transport": False,
    }))
    if workspace is not None:
        root = Path(workspace).resolve(strict=True)
        if not root.is_dir():
            raise ValueError("workspace must be a directory")

        def disk():
            usage = shutil.disk_usage(root)
            return {"total_bytes": usage.total, "used_bytes": usage.used,
                    "available_bytes": usage.free}
        registry.register(Capability("workspace.disk", "read", disk))
    return registry

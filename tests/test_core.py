import json
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from collections import namedtuple
from computer_link import Capability, Registry, diagnostics_registry
from computer_link.__main__ import respond


class CoreTests(unittest.TestCase):
    def test_independent_removal(self):
        r = diagnostics_registry()
        r.register(Capability("optional", "read", lambda: 1))
        r.remove("optional")
        self.assertTrue(r.call("system.summary")["os"])

    def test_no_mutation_in_read_only(self):
        r = Registry()
        called = []
        r.register(Capability("mutate", "write", lambda: called.append(1)))
        with self.assertRaises(PermissionError): r.call("mutate")
        self.assertEqual(called, [])

    def test_duplicate_and_effect(self):
        r = diagnostics_registry()
        with self.assertRaises(ValueError): r.register(Capability("system.summary", "read", lambda: 0))
        with self.assertRaises(ValueError): r.register(Capability("bad", "unknown", lambda: 0))

    def test_disk_no_path_disclosure(self):
        with tempfile.TemporaryDirectory() as d:
            r = diagnostics_registry(d)
            out = r.call("workspace.disk")
            self.assertGreater(out["total_bytes"], 0)
            self.assertNotIn(d, json.dumps(out))
            self.assertGreaterEqual(out["total_bytes"], out["used_bytes"])
            self.assertGreaterEqual(out["available_bytes"], 0)

    def test_reserved_blocks_preserved(self):
        usage = namedtuple("usage", "total used free")(100, 30, 60)
        with tempfile.TemporaryDirectory() as d, patch("computer_link.shutil.disk_usage", return_value=usage):
            self.assertEqual(diagnostics_registry(d).call("workspace.disk"), {"total_bytes":100,"used_bytes":30,"available_bytes":60})

    def test_no_implicit_workspace(self):
        self.assertEqual(len(diagnostics_registry().describe()), 1)

    def test_bad_requests(self):
        r = diagnostics_registry()
        for req in [[], {}, {"method": []}, {"method": "x"}, {"method": "system.summary", "params": {"shell": "whoami"}}, {"method": "capabilities.list", "params": {"x": 1}}, {"method": "system.summary", "id": []}]:
            self.assertIn("error", respond(r, req))

    def test_process_recovery(self):
        p = subprocess.run([sys.executable, "-m", "computer_link"], input='bad json\n{"id":2,"method":"system.summary"}\n', text=True, capture_output=True, check=True)
        results = [json.loads(x) for x in p.stdout.splitlines()]
        self.assertIn("error", results[0])
        self.assertEqual(results[1]["id"], 2)
        self.assertFalse(results[1]["result"]["network_listener"])

    def test_unknown_does_not_execute(self):
        out = respond(diagnostics_registry(), {"method":"shell.execute"})
        self.assertEqual(out["error"], "unsupported capability")

if __name__ == "__main__": unittest.main()

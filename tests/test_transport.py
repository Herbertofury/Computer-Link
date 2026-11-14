import json
import subprocess
import sys
import unittest


class TransportTests(unittest.TestCase):
    def test_invalid_adapter_output_recovers_without_partial_response(self):
        script = '''
from unittest.mock import patch
from computer_link import Capability, diagnostics_registry
from computer_link.__main__ import main
r = diagnostics_registry()
cycle = []
cycle.append(cycle)
outputs = [object(), cycle, float("nan"), float("inf"), -float("inf")]
for index, value in enumerate(outputs):
    r.register(Capability(str(index), "read", lambda value=value: value))
with patch("computer_link.__main__.diagnostics_registry", return_value=r):
    main()
'''
        requests = [{"id": i, "method": str(i)} for i in range(5)]
        requests.append({"id": 5, "method": "system.summary"})
        p = subprocess.run([sys.executable, "-c", script],
                           input="".join(json.dumps(r) + "\n" for r in requests),
                           text=True, capture_output=True, check=True)
        results = [json.loads(line) for line in p.stdout.splitlines()]
        self.assertEqual(len(results), 6)
        for i in range(5):
            self.assertEqual(results[i], {"id": i, "error": "invalid capability response"})
        self.assertIn("result", results[5])
        self.assertEqual(p.stderr, "")


if __name__ == "__main__":
    unittest.main()

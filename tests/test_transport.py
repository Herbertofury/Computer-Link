import io
import json
import subprocess
import sys
import unittest


class TransportTests(unittest.TestCase):
    def test_line_reads_are_bounded_and_boundary_is_preserved(self):
        from computer_link.__main__ import MAX_REQUEST_CHARS, bounded_lines
        class CheckedStream(io.StringIO):
            def readline(self, size=-1):
                if size != MAX_REQUEST_CHARS + 1:
                    raise AssertionError("unbounded read")
                return super().readline(size)
        exact = "x" * (MAX_REQUEST_CHARS - 1) + "\n"
        stream = CheckedStream(exact + "y" * (MAX_REQUEST_CHARS * 4) + "\nvalid\n")
        self.assertEqual(list(bounded_lines(stream)), [exact, None, "valid\n"])
        self.assertEqual(list(bounded_lines(CheckedStream("x" * MAX_REQUEST_CHARS))),
                         ["x" * MAX_REQUEST_CHARS])

    def test_oversized_request_is_rejected_and_next_line_recovers(self):
        # The current diagnostics protocol has no bulk-payload methods.
        oversized = json.dumps({"id": "x" * 70000, "method": "system.summary"})
        p = subprocess.run([sys.executable, "-m", "computer_link"],
                           input=oversized + "\n" + '{"id":2,"method":"system.summary"}\n',
                           text=True, capture_output=True, check=True)
        results = [json.loads(line) for line in p.stdout.splitlines()]
        self.assertEqual(results[0], {"error": "request too large"})
        self.assertEqual(results[1]["id"], 2)
        self.assertIn("result", results[1])

    def test_oversized_eof_is_one_error(self):
        p = subprocess.run([sys.executable, "-m", "computer_link"],
                           input="x" * 200000, text=True, capture_output=True, check=True)
        self.assertEqual(p.stdout, '{"error": "request too large"}\n')


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

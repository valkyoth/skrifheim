"""Positive and negative controls for focused assurance completion."""
import subprocess
import sys
import unittest
from unittest.mock import patch
import assurance


class CompletionTests(unittest.TestCase):
    def test_selectors(self):
        self.assertEqual(len(assurance.selected("all")), 6)
        with self.assertRaisesRegex(ValueError, "unknown"):
            assurance.selected("typo")

    def test_corpus_requires_all_nonzero_cases(self):
        assurance.require_cases("wal_header: 4 cases\n", ["wal_header"])
        for text in ["", "wal_header: 0 cases", "wal_header: 4 ignored",
                     "wrong: 4 cases", "wal_header: 4 cases\nwal_header: 4 cases"]:
            with self.assertRaises(ValueError):
                assurance.require_cases(text, ["wal_header"])

    def test_fuzz_requires_completion_and_coverage(self):
        assurance.require_fuzz("#1000 DONE cov: 25\nDone 1000 runs in 1 second(s)\n", 1000)
        for text in ["", "Done 0 runs in 1 second(s)", "Done 1000 runs in 1 second(s)",
                     "cov: 12\nDone 2 runs in 1 second(s)", "cov: 12\nTIMEOUT"]:
            with self.assertRaises(ValueError):
                assurance.require_fuzz(text, 1000)

    def test_command_failure_is_not_a_pass(self):
        with self.assertRaises(RuntimeError):
            assurance.run([sys.executable, "-c", "raise SystemExit(3)"])

    def test_timeout_is_not_a_pass(self):
        with self.assertRaises(subprocess.TimeoutExpired):
            assurance.run([sys.executable, "-c", "import time; time.sleep(10)"], timeout=0.05)

    def test_smoke_command_failure_propagates(self):
        with patch.object(sys, "argv", ["assurance.py", "smoke"]), \
                patch.object(assurance, "run", side_effect=RuntimeError("failed")):
            with self.assertRaises(RuntimeError):
                assurance.main()


if __name__ == "__main__":
    unittest.main()

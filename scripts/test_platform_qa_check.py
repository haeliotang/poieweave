# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later

import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

import platform_qa_check

def receipt(checks, signing="unsigned"):
    return {
        "schema": "poie.platform-qa/0.1",
        "version": "0.4.5",
        "platform": "macos-aarch64",
        "signingStatus": signing,
        "checks": [{"id": item, "status": "pass", "evidence": "manual"} for item in checks],
    }

def all_required():
    return ["install", "launch", "gatekeeper_notice", "dock_icon", "data_migration", "version_consistency", "sha256sums", "release_notes_signing_status", "no_telemetry"]

class PlatformQaTests(unittest.TestCase):
    def test_complete_receipt_passes(self):
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "qa.json"
            path.write_text(json.dumps(receipt(all_required())), encoding="utf-8")
            out = io.StringIO()
            with redirect_stdout(out):
                code = platform_qa_check.main(["--input", str(path)])
            self.assertEqual(code, 0)
            self.assertTrue(json.loads(out.getvalue())["ok"])

    def test_missing_check_fails(self):
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "qa.json"
            path.write_text(json.dumps(receipt(all_required()[:-1])), encoding="utf-8")
            with redirect_stdout(io.StringIO()):
                code = platform_qa_check.main(["--input", str(path)])
            self.assertEqual(code, 1)

    def test_unknown_signing_status_fails(self):
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "qa.json"
            path.write_text(json.dumps(receipt(all_required(), signing="trust-me")), encoding="utf-8")
            with redirect_stdout(io.StringIO()):
                code = platform_qa_check.main(["--input", str(path)])
            self.assertEqual(code, 1)

if __name__ == "__main__":
    unittest.main()

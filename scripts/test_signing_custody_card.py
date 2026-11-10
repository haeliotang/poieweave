# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later

import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

import signing_custody_card


class SigningCustodyCardTests(unittest.TestCase):
    def test_card_contains_digest_and_no_payload(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            archive = root / "signing.edgea"
            archive.write_bytes(b"ENCRYPTED-ARCHIVE-PAYLOAD")
            card = root / "CARD.md"
            buffer = io.StringIO()
            with redirect_stdout(buffer):
                code = signing_custody_card.main([
                    "--archive", str(archive),
                    "--keychain-service", "com.example.signing",
                    "--keychain-account", "backup-1",
                    "--passphrase-location", "sealed envelope at home",
                    "--output", str(card),
                    "--created", "2026-11-06",
                ])
            self.assertEqual(code, 0)
            text = card.read_text(encoding="utf-8")
            self.assertIn("signing.edgea", text)
            self.assertIn(signing_custody_card.sha256(archive), text)
            self.assertNotIn("ENCRYPTED-ARCHIVE-PAYLOAD", text)
            self.assertEqual(json.loads(buffer.getvalue())["ok"], True)

    def test_digest_mismatch_fails(self):
        with tempfile.TemporaryDirectory() as raw:
            archive = Path(raw) / "a.bin"
            archive.write_bytes(b"x")
            with redirect_stdout(io.StringIO()):
                code = signing_custody_card.main([
                    "--archive", str(archive),
                    "--keychain-service", "s", "--keychain-account", "a",
                    "--passphrase-location", "x", "--output", str(Path(raw) / "c.md"),
                    "--expected-sha256", "0" * 64,
                ])
            self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()

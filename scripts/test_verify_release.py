import hashlib
import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

import verify_release


class VerifyReleaseTests(unittest.TestCase):
    def test_receipt_accepts_matching_artifacts(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "Poieweave_0.3.1_universal.app.zip").write_bytes(b"app")
            digest = hashlib.sha256(b"app").hexdigest()
            (root / "SHA256SUMS").write_text(f"{digest}  Poieweave_0.3.1_universal.app.zip\n", encoding="utf-8")
            out = root / "receipt.json"
            buffer = io.StringIO()
            with redirect_stdout(buffer):
                code = verify_release.main([
                    "--artifacts", str(root), "--expected-version", "0.3.1", "--output", str(out)
                ])
            self.assertEqual(code, 0)
            receipt = json.loads(out.read_text(encoding="utf-8"))
            self.assertTrue(receipt["ok"])
            self.assertEqual(receipt["sha256Sums"], 1)

    def test_receipt_fails_on_digest_mismatch(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "Poieweave_0.3.1_universal.app.zip").write_bytes(b"app")
            (root / "SHA256SUMS").write_text("0" * 64 + "  Poieweave_0.3.1_universal.app.zip\n", encoding="utf-8")
            with redirect_stdout(io.StringIO()):
                code = verify_release.main(["--artifacts", str(root), "--expected-version", "0.3.1"])
            self.assertEqual(code, 1)

    def test_receipt_fails_on_unlisted_artifact(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "Poieweave_0.3.1_universal.app.zip").write_bytes(b"app")
            (root / "other.bin").write_bytes(b"other")
            (root / "SHA256SUMS").write_text("", encoding="utf-8")
            with redirect_stdout(io.StringIO()):
                code = verify_release.main(["--artifacts", str(root), "--expected-version", "0.3.1"])
            self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()

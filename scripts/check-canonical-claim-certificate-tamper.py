#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tamper test: modified expected/claims/engine hashes must fail verification."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "docs/research/canonical-claim-certificate-index.json"
VERIFIER = ROOT / "scripts/verify-canonical-claim-certificates.py"


def rejected(cert: dict) -> bool:
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
        json.dump(cert, handle)
        path = Path(handle.name)
    try:
        result = subprocess.run(
            [sys.executable, str(VERIFIER), "--certificate", str(path)],
            capture_output=True,
            text=True,
            check=False,
        )
        return result.returncode != 0
    finally:
        path.unlink(missing_ok=True)


def main() -> int:
    index = json.loads(INDEX.read_text())
    entry = index["certificates"][0]
    original = json.loads((ROOT / entry["certificate"]).read_text())
    if rejected(original):
        print("tamper test: untouched certificate was rejected", file=sys.stderr)
        return 1
    tampered = json.loads(json.dumps(original))
    tampered["experiment"]["expected_sha256"] = "0" * 64
    if not rejected(tampered):
        print("tamper test: expected hash mutation not detected", file=sys.stderr)
        return 1
    tampered = json.loads(json.dumps(original))
    tampered["claims"] = [original["claims"][0], "C999"]
    if not rejected(tampered):
        print("tamper test: claim mutation not detected", file=sys.stderr)
        return 1
    tampered = json.loads(json.dumps(original))
    tampered["engine"]["sha256"] = "0" * 64
    if not rejected(tampered):
        print("tamper test: engine hash mutation not detected", file=sys.stderr)
        return 1
    print("ok canonical claim certificate tamper: 3 mutations rejected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

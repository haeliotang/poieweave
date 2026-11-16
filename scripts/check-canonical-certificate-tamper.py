#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify canonical certificates pass and tampered copies fail closed."""
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    certs = sorted((ROOT / "experiments").glob("certificate-s2-canonical-*/certificate.json"))
    if not certs:
        print("no canonical certificates", file=sys.stderr)
        return 1
    for cert in certs:
        verifier = cert.parent / "verify.py"
        ok = subprocess.run([sys.executable, str(verifier), str(cert)], capture_output=True, text=True)
        if ok.returncode != 0:
            print(f"FAIL original {cert.parent.name}: {ok.stdout} {ok.stderr}", file=sys.stderr)
            return 1
        data = json.loads(cert.read_text())
        data["experiment"]["expected_sha256"] = "0" * 64
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            json.dump(data, handle)
            tampered = Path(handle.name)
        try:
            bad = subprocess.run([sys.executable, str(verifier), str(tampered)], capture_output=True, text=True)
        finally:
            tampered.unlink(missing_ok=True)
        if bad.returncode == 0:
            print(f"FAIL tamper accepted {cert.parent.name}", file=sys.stderr)
            return 1
    print(f"ok canonical certificate tamper: {len(certs)} certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

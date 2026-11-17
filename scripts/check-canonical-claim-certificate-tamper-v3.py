#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tamper tests for v3 canonical claim certificates."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v3.json"
VERIFIER = ROOT / "scripts/verify-canonical-claim-certificates-v3.py"


def rejected(cert: dict) -> bool:
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
        json.dump(cert, handle)
        path = Path(handle.name)
    try:
        return subprocess.run([sys.executable, str(VERIFIER), "--certificate", str(path)],
                              capture_output=True, text=True, check=False).returncode != 0
    finally:
        path.unlink(missing_ok=True)


def main() -> int:
    entry = json.loads(INDEX.read_text())["certificates"][0]
    original = json.loads((ROOT / entry["certificate"]).read_text())
    if rejected(original):
        print("tamper: untouched v3 certificate rejected", file=sys.stderr)
        return 1
    mutations = [
        ("expected hash", lambda c: c["experiment"].__setitem__("expected_sha256", "0" * 64)),
        ("environment hash", lambda c: c["source_environment"]["files"][0].__setitem__("sha256", "0" * 64)),
        ("engine hash", lambda c: c["engine"].__setitem__("sha256", "0" * 64)),
        ("model hash", lambda c: c["models"][0].__setitem__("sha256", "0" * 64)),
        ("actual replay", lambda c: c["replay"].__setitem__("output_sha256", "0" * 64)),
        ("claim", lambda c: c["claims"].append("C999")),
        ("predecessor", lambda c: c["predecessor"].__setitem__("sha256", "0" * 64)),
        ("predecessor relation", lambda c: c["predecessor"].__setitem__("relation", "EXECUTION_SEMANTIC_CHANGE")),
        ("native evidence", lambda c: c["native_observatory"].__setitem__("sha256", "0" * 64)),
        ("scope", lambda c: c.__setitem__("scope", "unbounded")),
        ("v2 masquerade", lambda c: c.__setitem__("schema", "poie.canonical-s2-certificate/3")),
    ]
    for name, mutate in mutations:
        cert = json.loads(json.dumps(original))
        mutate(cert)
        if not rejected(cert):
            print(f"tamper: mutation not detected: {name}", file=sys.stderr)
            return 1
    print(f"ok v3 certificate tamper: {len(mutations)} mutations rejected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

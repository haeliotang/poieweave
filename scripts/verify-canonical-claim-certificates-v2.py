#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify v2 canonical claim certificates and their full runtime environment."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v2.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(cert: dict, ledger: str, label: str) -> list[str]:
    problems = []
    if cert.get("schema") != "poie.canonical-s2-certificate/3":
        problems.append("schema")
    if cert.get("status") != "CANONICAL_ISSUED_V2":
        problems.append("status")
    if cert.get("verifier") != "poie-canonical-s2-certificate/3":
        problems.append("verifier")
    if sha(ROOT / "docs/research/claims-ledger.zh-CN.md") != cert.get("claims_sha256"):
        problems.append("claims hash")
    for claim in cert.get("claims", []):
        if f"| {claim} |" not in ledger:
            problems.append(f"claim {claim}")
    for path, expected in ((cert["engine"]["source"], cert["engine"]["sha256"]),
                           (cert["runner"]["source"], cert["runner"]["sha256"]),
                           (cert["experiment"]["expected"], cert["experiment"]["expected_sha256"])):
        if sha(ROOT / path) != expected:
            problems.append(path)
    for entry in cert.get("models", []):
        if sha(ROOT / entry["path"]) != entry["sha256"]:
            problems.append(entry["path"])
    for entry in cert.get("source_environment", {}).get("files", []):
        if sha(ROOT / entry["path"]) != entry["sha256"]:
            problems.append(entry["path"])
    predecessor = cert.get("predecessor")
    if predecessor:
        path = ROOT / predecessor["path"]
        if not path.is_file() or sha(path) != predecessor["sha256"]:
            problems.append("predecessor")
    if not cert.get("scope") or not cert.get("semantics"):
        problems.append("scope/semantics")
    return problems


def main(argv: list[str]) -> int:
    ledger = (ROOT / "docs/research/claims-ledger.zh-CN.md").read_text()
    if "--certificate" in argv:
        path = Path(argv[argv.index("--certificate") + 1])
        problems = verify(json.loads(path.read_text()), ledger, str(path))
        if problems:
            print("v2 certificate FAIL: " + ", ".join(problems), file=sys.stderr)
            return 1
        print("v2 certificate ok")
        return 0
    index = json.loads(INDEX.read_text())
    problems = []
    for entry in index["certificates"]:
        problems.extend(verify(json.loads((ROOT / entry["certificate"]).read_text()), ledger, entry["claim"]))
    if problems:
        print("v2 certificates FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok v2 canonical claim certificates verified: {len(index['certificates'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

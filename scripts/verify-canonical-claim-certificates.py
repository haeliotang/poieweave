#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify canonical claim certificates against current repository bytes."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "docs/research/canonical-claim-certificate-index.json"
SCHEMA = "poie.canonical-s2-certificate/2"
VERIFIER = "poie-canonical-s2-certificate/2"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_certificate(cert: dict, ledger_text: str, label: str) -> list[str]:
    problems: list[str] = []
    if cert.get("schema") != SCHEMA:
        problems.append(f"{label}: schema")
    if cert.get("status") != "CANONICAL_ISSUED":
        problems.append(f"{label}: status")
    if cert.get("verifier") != VERIFIER:
        problems.append(f"{label}: verifier")
    if sha256(ROOT / "docs/research/claims-ledger.zh-CN.md") != cert.get("claims_sha256"):
        problems.append(f"{label}: claims hash")
    for claim in cert.get("claims", []):
        if f"| {claim} |" not in ledger_text:
            problems.append(f"{label}: missing claim {claim}")
    engine = ROOT / cert["engine"]["source"]
    if sha256(engine) != cert["engine"]["sha256"]:
        problems.append(f"{label}: engine hash")
    runner = ROOT / cert["runner"]["source"]
    if sha256(runner) != cert["runner"]["sha256"]:
        problems.append(f"{label}: runner hash")
    expected = ROOT / cert["experiment"]["expected"]
    if sha256(expected) != cert["experiment"]["expected_sha256"]:
        problems.append(f"{label}: expected hash")
    for model in cert.get("models", []):
        if sha256(ROOT / model["path"]) != model["sha256"]:
            problems.append(f"{label}: model hash {model['path']}")
    for source in cert.get("evidence_sources", []):
        if sha256(ROOT / source["path"]) != source["sha256"]:
            problems.append(f"{label}: evidence source {source['path']}")
    if not cert.get("scope") or not cert.get("semantics"):
        problems.append(f"{label}: scope/semantics")
    return problems


def main(argv: list[str]) -> int:
    ledger_text = (ROOT / "docs/research/claims-ledger.zh-CN.md").read_text()
    problems: list[str] = []
    if "--certificate" in argv:
        path = Path(argv[argv.index("--certificate") + 1])
        problems = verify_certificate(json.loads(path.read_text()), ledger_text, str(path))
        if problems:
            print("canonical claim certificate FAIL: " + ", ".join(problems), file=sys.stderr)
            return 1
        print("canonical claim certificate ok")
        return 0
    index = json.loads(INDEX.read_text())
    for entry in index["certificates"]:
        cert = json.loads((ROOT / entry["certificate"]).read_text())
        problems.extend(verify_certificate(cert, ledger_text, entry["claim"]))
    if problems:
        print("canonical claim certificates FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok canonical claim certificates verified: {len(index['certificates'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

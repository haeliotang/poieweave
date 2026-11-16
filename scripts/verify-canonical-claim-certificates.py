#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify every canonical claim certificate against current repository bytes."""

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


def main() -> int:
    index = json.loads(INDEX.read_text())
    claims_ledger = ROOT / "docs/research/claims-ledger.zh-CN.md"
    ledger_text = claims_ledger.read_text()
    problems: list[str] = []
    for entry in index["certificates"]:
        path = ROOT / entry["certificate"]
        cert = json.loads(path.read_text())
        label = entry["claim"]
        if cert.get("schema") != SCHEMA:
            problems.append(f"{label}: schema")
        if cert.get("status") != "CANONICAL_ISSUED":
            problems.append(f"{label}: status")
        if cert.get("verifier") != VERIFIER:
            problems.append(f"{label}: verifier")
        if sha256(claims_ledger) != cert.get("claims_sha256"):
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
    if problems:
        print("canonical claim certificates FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok canonical claim certificates verified: {len(index['certificates'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

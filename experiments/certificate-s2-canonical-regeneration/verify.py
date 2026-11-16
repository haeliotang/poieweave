#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify one canonical S2 certificate."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = "poie.canonical-s2-certificate/1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(path: str) -> int:
    cert = json.loads(Path(path).read_text())
    problems: list[str] = []
    if cert.get("schema") != SCHEMA:
        problems.append("schema")
    if cert.get("status") != "REISSUED":
        problems.append("status")
    if cert.get("verifier") != "poie-canonical-s2-certificate/1":
        problems.append("verifier")
    source = ROOT / cert.get("source_certificate", "")
    if not source.is_file():
        problems.append("source certificate path")
    elif sha256(source) != cert.get("source_certificate_sha256"):
        problems.append("source certificate hash")
    for model in cert.get("models", []):
        model_path = ROOT / model.get("path", "")
        if not model_path.is_file():
            problems.append(f"model path {model.get('path')}")
        elif sha256(model_path) != model.get("sha256"):
            problems.append(f"model hash {model.get('path')}")
    engine = ROOT / cert.get("engine", {}).get("source", "")
    if not engine.is_file():
        problems.append("engine source")
    elif sha256(engine) != cert["engine"].get("sha256"):
        problems.append("engine hash")
    runner = ROOT / cert.get("runner", {}).get("source", "")
    if not runner.is_file():
        problems.append("runner source")
    elif sha256(runner) != cert["runner"].get("sha256"):
        problems.append("runner hash")
    expected = ROOT / cert.get("experiment", {}).get("expected", "")
    if not expected.is_file():
        problems.append("expected path")
    elif sha256(expected) != cert["experiment"].get("expected_sha256"):
        problems.append("expected hash")
    else:
        marker = cert.get("result_marker", "")
        if not marker or marker not in expected.read_text():
            problems.append("result marker")
    claims = ROOT / "docs/research/claims-ledger.zh-CN.md"
    if not claims.is_file() or sha256(claims) != cert.get("claims_sha256"):
        problems.append("claims hash")
    else:
        text = claims.read_text()
        for claim in cert.get("claims", []):
            if f"| {claim} |" not in text:
                problems.append(f"missing claim {claim}")
    if problems:
        print(f"certificate=FAIL problems={problems}")
        return 1
    print("certificate=ok")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: verify.py <certificate.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

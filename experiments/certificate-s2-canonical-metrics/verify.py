#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the scoped canonical metrics certificate."""
from __future__ import annotations
import hashlib, json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
def main(path: str) -> int:
    cert = json.loads(Path(path).read_text())
    problems = []
    if cert.get("schema") != "poie.canonical-s2-metrics-certificate/1":
        problems.append("schema")
    if cert.get("status") != "REISSUED":
        problems.append("status")
    if cert.get("coverage_definition", {}).get("id") != "service-impact-v1":
        problems.append("coverage definition")
    if cert.get("coverage_definition", {}).get("canonical_value") != "9/10 service-impact":
        problems.append("coverage value")
    source = ROOT / cert.get("source_certificate", "")
    if not source.is_file() or sha(source) != cert.get("source_certificate_sha256"):
        problems.append("source certificate")
    for model in cert.get("models", []):
        model_path = ROOT / model.get("path", "")
        if not model_path.is_file() or sha(model_path) != model.get("sha256"):
            problems.append(f"model {model.get('path')}")
    engine = ROOT / cert.get("engine", {}).get("source", "")
    if not engine.is_file() or sha(engine) != cert["engine"].get("sha256"):
        problems.append("engine")
    runner = ROOT / cert.get("runner", {}).get("source", "")
    if not runner.is_file() or sha(runner) != cert["runner"].get("sha256"):
        problems.append("runner")
    expected = ROOT / cert.get("experiment", {}).get("expected", "")
    if not expected.is_file() or sha(expected) != cert.get("experiment", {}).get("expected_sha256"):
        problems.append("expected")
    else:
        text = expected.read_text()
        for marker in cert.get("result_markers", []):
            if marker not in text:
                problems.append(f"marker {marker[:20]}")
    if sha(ROOT / "docs/research/claims-ledger.zh-CN.md") != cert.get("claims_sha256"):
        problems.append("claims hash")
    if problems:
        print(f"certificate=FAIL problems={problems}")
        return 1
    print("certificate=ok")
    return 0
if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

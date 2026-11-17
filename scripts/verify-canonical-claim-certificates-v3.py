#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify v3 canonical claim certificates, native evidence, replay and predecessors."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v3.json"
V2_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v2.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replay_evidence() -> dict:
    result = subprocess.run(["python3", "scripts/check-experiment-replays.py"], cwd=ROOT, capture_output=True, check=False)
    output = result.stdout + result.stderr
    return {"command": "python3 scripts/check-experiment-replays.py",
            "output_sha256": hashlib.sha256(output).hexdigest(), "exit_code": result.returncode}


def verify(cert: dict, ledger: str, replay: dict, v2_claims: dict[str, str]) -> list[str]:
    problems = []
    if cert.get("schema") != "poie.canonical-s2-certificate/4":
        problems.append("schema")
    if cert.get("version") != 3:
        problems.append("version")
    if cert.get("status") != "CANONICAL_ISSUED_V3":
        problems.append("status")
    if cert.get("verifier") != "poie-canonical-s2-certificate/4":
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
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            problems.append(entry["path"])
    predecessor = cert.get("predecessor", {})
    v2_path = predecessor.get("path")
    if not v2_path or v2_path not in v2_claims:
        problems.append("predecessor v2 not in v2 index")
    elif sha(ROOT / v2_path) != predecessor.get("sha256") or predecessor.get("sha256") != v2_claims[v2_path]:
        problems.append("predecessor v2 hash")
    if predecessor.get("relation") != "OBSERVATION_ONLY_SUCCESSOR":
        problems.append("predecessor relation")
    native = cert.get("native_observatory", {})
    if not native.get("path") or sha(ROOT / native["path"]) != native.get("sha256"):
        problems.append("native observatory evidence")
    if cert.get("replay") != replay:
        problems.append("actual replay evidence")
    scope = cert.get("scope", "")
    if not scope.startswith("model/simulation evidence only; canonical claim "):
        problems.append("scope")
    if "native read-only AssetStatus observation does not alter execution" not in cert.get("semantics", ""):
        problems.append("semantics")
    return problems


def main(argv: list[str]) -> int:
    ledger = (ROOT / "docs/research/claims-ledger.zh-CN.md").read_text()
    v2_index = json.loads(V2_INDEX.read_text())
    v2_claims = {entry["certificate"]: sha(ROOT / entry["certificate"]) for entry in v2_index["certificates"]}
    replay = replay_evidence()
    if "--certificate" in argv:
        path = Path(argv[argv.index("--certificate") + 1])
        problems = verify(json.loads(path.read_text()), ledger, replay, v2_claims)
        if problems:
            print("v3 certificate FAIL: " + ", ".join(problems), file=sys.stderr)
            return 1
        print("v3 certificate ok")
        return 0
    index = json.loads(INDEX.read_text())
    problems = []
    for entry in index["certificates"]:
        problems.extend(verify(json.loads((ROOT / entry["certificate"]).read_text()), ledger, replay, v2_claims))
    if problems:
        print("v3 certificates FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok v3 canonical claim certificates verified: {len(index['certificates'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

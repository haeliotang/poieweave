#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Adversarial evidence-integrity controls for N8-H claims and manifests."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
VERIFIER = PHASE / "verify_evidence.py"
MANIFEST = PHASE / "manifest.json"
SIDECAR = PHASE / "manifest.sha256"
CLAIMS = PHASE / "comparisons/strict-trajectories.json"


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, str(VERIFIER), *args], cwd=ROOT, capture_output=True, text=True)


def rejected(*args: str) -> bool:
    return run(*args).returncode != 0


results = []


def record(name: str, detected: bool) -> None:
    results.append({"control": name, "detected": detected})
    print(name, "PASS" if detected else "FAIL")


if run("--frozen", "--claim").returncode != 0:
    print("positive frozen/claim control rejected", file=sys.stderr)
    raise SystemExit(1)

# Raw CSV tamper
csv_path = PHASE / "external/N8H-N8-01.csv"
original = csv_path.read_text()
try:
    csv_path.write_text(original.replace("144000000.000000000", "144000001.000000000", 1))
    record("raw-csv-tamper", rejected("--frozen"))
finally:
    csv_path.write_text(original)

# Manifest content tamper (entry hash)
manifest = json.loads(MANIFEST.read_text())
original_manifest_text = MANIFEST.read_text()
try:
    manifest["files"][0]["sha256"] = "0" * 64
    MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n")
    record("manifest-content-tamper", rejected("--frozen"))
finally:
    MANIFEST.write_text(original_manifest_text)

# Manifest digest sidecar tamper
original_sidecar = SIDECAR.read_text()
try:
    SIDECAR.write_text("0" * 64 + "\n")
    record("manifest-digest-tamper", rejected("--frozen"))
finally:
    SIDECAR.write_text(original_sidecar)

# Missing evidence file
missing = PHASE / "external/N8H-N8-04.csv"
original_missing = missing.read_text()
try:
    missing.unlink()
    record("missing-evidence-file", rejected("--frozen"))
finally:
    missing.write_text(original_missing)

# Wrong execution batch
head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
with tempfile.TemporaryDirectory() as directory:
    execution = Path(directory) / "execution.json"
    execution.write_text(json.dumps({
        "schema": "poie.s5v-n8h-execution-evidence/1",
        "kind": "CURRENT_RUN_EXECUTION_EVIDENCE",
        "git_head": "0" * 40,
        "files": [{"path": str(CLAIMS.relative_to(ROOT)), "sha256": "0" * 64}],
    }) + "\n")
    record("wrong-execution-batch", rejected("--execution", str(execution), "--expected-head", head))

# Claim inconsistency: MATCH with findings
claim = json.loads(CLAIMS.read_text())
original_claim_text = CLAIMS.read_text()
try:
    claim["scenarios"][0]["findings"] = [{"class": "UNEXPLAINED_RESIDUAL"}]
    claim["scenarios"][0]["status"] = "MATCH"
    CLAIMS.write_text(json.dumps(claim, indent=2) + "\n")
    record("claim-inconsistency", rejected("--claim"))
finally:
    CLAIMS.write_text(original_claim_text)

output = PHASE / "negative-controls" / "integrity-results.json"
output.write_text(json.dumps({"schema": "poie.s5v-n8h-integrity-controls/1", "results": results}, indent=2) + "\n")
raise SystemExit(0 if all(entry["detected"] for entry in results) else 1)

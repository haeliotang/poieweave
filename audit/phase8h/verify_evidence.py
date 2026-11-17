#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent N8-H evidence-integrity verifier for frozen/current-run evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
MANIFEST = PHASE / "manifest.json"
SIDECAR = PHASE / "manifest.sha256"
OLD = PHASE / "old-evidence.json"
CLAIMS = PHASE / "comparisons/strict-trajectories.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_frozen() -> list[str]:
    problems = []
    if not SIDECAR.is_file():
        return ["missing manifest.sha256"]
    expected_manifest_sha = SIDECAR.read_text().strip().split()[0]
    if sha(MANIFEST) != expected_manifest_sha:
        problems.append("manifest digest mismatch")
    manifest = json.loads(MANIFEST.read_text())
    if manifest.get("kind") != "FROZEN_TRACKED_EVIDENCE":
        problems.append("manifest kind")
    for entry in manifest["files"]:
        path = ROOT / entry["path"]
        if not path.is_file():
            problems.append(f"missing {entry['path']}")
        elif sha(path) != entry["sha256"]:
            problems.append(f"hash {entry['path']}")
    old = json.loads(OLD.read_text())
    for entry in old["files"]:
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            problems.append(f"old evidence {entry['path']}")
    return problems


def verify_execution(path: Path, expected_head: str | None) -> list[str]:
    problems = []
    data = json.loads(path.read_text())
    if data.get("kind") != "CURRENT_RUN_EXECUTION_EVIDENCE":
        problems.append("execution kind")
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    if expected_head is not None and data.get("git_head") != expected_head:
        problems.append("wrong execution batch")
    if expected_head is None and data.get("git_head") != head:
        problems.append("execution git head")
    for entry in data.get("files", []):
        path_entry = ROOT / entry["path"]
        if not path_entry.is_file() or sha(path_entry) != entry["sha256"]:
            problems.append(f"execution hash {entry['path']}")
    return problems


def verify_claim(path: Path) -> list[str]:
    problems = []
    data = json.loads(path.read_text())
    for scenario in data.get("scenarios", []):
        findings = scenario.get("findings", [])
        status = scenario.get("status")
        if status == "MATCH" and findings:
            problems.append(f"{scenario.get('scenario')}: MATCH with findings")
        if status != "MATCH" and not findings:
            problems.append(f"{scenario.get('scenario')}: failure without findings")
    if not data.get("scenarios"):
        problems.append("empty claim set")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen", action="store_true")
    parser.add_argument("--execution")
    parser.add_argument("--expected-head")
    parser.add_argument("--claim", nargs="?", const=str(CLAIMS))
    args = parser.parse_args()
    problems = []
    if args.frozen:
        problems.extend(verify_frozen())
    if args.execution:
        problems.extend(verify_execution(Path(args.execution), args.expected_head))
    if args.claim:
        problems.extend(verify_claim(Path(args.claim)))
    if problems:
        print("N8-H evidence integrity FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print("ok N8-H evidence integrity")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check that the Phase 1 audit artifacts are internally consistent."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    report = json.loads((HERE / "report.json").read_text())
    matrix = json.loads((HERE / "matrix.json").read_text())
    problems = []
    if report["gate1"] != "PASS_FOR_EXTERNAL_MODEL_PHASE":
        problems.append("gate1")
    if set(report["counts"]) - {"VERIFIED", "VERIFIED_WITH_LIMITATIONS"}:
        problems.append("non-passing status")
    if report["all_certificate_hash_checks"]["failures"]:
        problems.append("certificate hash failures")
    for row in report["results"]:
        evidence = ROOT / row["evidence"]
        if not evidence.is_file() or sha(evidence) != row["evidence_sha256"]:
            problems.append(f"evidence {row['claim']}")
    if len(matrix["rows"]) != len(report["results"]):
        problems.append("matrix rows")
    if problems:
        print("phase1 audit artifacts FAIL: " + ", ".join(problems))
        return 1
    print("ok phase1 audit artifacts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

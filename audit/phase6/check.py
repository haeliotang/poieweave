#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify N6-A resource lifetime evidence."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    problems = []
    manifest = json.loads((PHASE / "manifest.json").read_text())
    for entry in manifest["files"]:
        if sha(ROOT / entry["path"]) != entry["sha256"]:
            problems.append(f"hash: {entry['path']}")
    for path, key in ((manifest["tools"]["external_fmu"], "external_fmu_sha256"),):
        if sha(ROOT / path) != manifest["tools"][key]:
            problems.append(f"tool hash: {path}")
    comparison = json.loads((PHASE / "differences/resource-lifetime.json").read_text())
    if len(comparison["cases"]) != 5:
        problems.append("boundary case count")
    for entry in comparison["cases"]:
        if entry["status"] != "EXPLAINED":
            problems.append(f"{entry['case']}: status")
        if entry["contract"]["step_level_release"] != "NOT_REPRESENTABLE":
            problems.append(f"{entry['case']}: step-level claim")
    for failure in (7199, 7200, 7201, 9000):
        audit = json.loads((PHASE / "canonical" / f"N6-A-{failure}-audit.json").read_text())
        if not audit["repair_interrupted"] or audit["restoration_committed"]:
            problems.append(f"N6-A-{failure}: lifetime semantics")
    final = json.loads((PHASE / "canonical/N6-A-10800-audit.json").read_text())
    if final["repair_interrupted"] or not final["restoration_committed"]:
        problems.append("N6-A-10800: commit boundary")
    negatives = json.loads((PHASE / "negative-controls/lifetime-results.json").read_text())
    if not all(entry["detected"] for entry in negatives["results"]):
        problems.append("negative controls")
    if problems:
        print("N6-A evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N6-A lifetime: {len(comparison['cases'])} boundaries, {len(negatives['results'])} negative controls")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

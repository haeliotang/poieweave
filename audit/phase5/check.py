#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify N5 evidence, contracts and claim invariants."""

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
    for path, key in ((manifest["tools"]["external_fmu"], "external_fmu_sha256"),
                      ("external/fmu-n5-scheduler/src/model.c", "model_source_sha256"),
                      ("external/fmu-n5-scheduler/src/harness.c", "harness_source_sha256")):
        if sha(ROOT / path) != manifest["tools"][key]:
            problems.append(f"tool hash: {path}")
    config = json.loads((PHASE / "config.json").read_text())
    comparison = json.loads((PHASE / "differences/comparison.json").read_text())
    if {entry["case"] for entry in comparison["cases"]} != {entry["id"] for entry in config["cases"]}:
        problems.append("case set")
    for entry in comparison["cases"]:
        if not entry["cross_model_agreement"]:
            problems.append(f"{entry['case']}: cross-model agreement")
        if not entry["contract"]["canonical_valid"] or not entry["contract"]["external_valid"]:
            problems.append(f"{entry['case']}: contract")
    for case in config["cases"]:
        summary = json.loads((PHASE / "canonical" / f"{case['id']}-summary.json").read_text())
        if case.get("expected_interrupted"):
            if not summary["interrupted"] or summary["restored_targets"]:
                problems.append(f"{case['id']}: canonical interruption")
        else:
            if summary["interrupted"]:
                problems.append(f"{case['id']}: canonical unexpected interruption")
            if abs(summary["makespan_s"] - case["expected_makespan_s"]) > 1e-6:
                problems.append(f"{case['id']}: makespan")
            if set(summary["restored_targets"]) != {config["task_contract"][name]["target"] for name in case["tasks"]}:
                problems.append(f"{case['id']}: restoration")
    negatives = json.loads((PHASE / "negative-controls/results.json").read_text())
    if not negatives["results"] or not all(entry["detected"] for entry in negatives["results"]):
        problems.append("negative controls")
    importer = json.loads((PHASE / "scheduling/importer-agreement.json").read_text())
    if not all(entry["ok"] for entry in importer["results"]):
        problems.append("importer agreement")
    if problems:
        print("N5 evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N5 evidence: {len(config['cases'])} cases, {len(negatives['results'])} negative controls")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

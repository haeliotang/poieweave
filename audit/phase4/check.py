#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the N4 evidence package and claim invariants."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
MANIFEST = PHASE / "manifest.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    problems = []
    manifest = json.loads(MANIFEST.read_text())
    for entry in manifest["files"]:
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            problems.append(f"hash: {entry['path']}")
    tools = manifest["tools"]
    for path, key in ((tools["external_fmu"], "external_fmu_sha256"),
                      ("external/fmu-n4-pv-battery/src/model.c", "model_source_sha256"),
                      ("external/fmu-n4-pv-battery/src/harness.c", "harness_source_sha256"),
                      (tools["repair_fmu"], "repair_fmu_sha256"),
                      ("external/fmu-n4-repair/src/model.c", "repair_model_source_sha256")):
        if sha(ROOT / path) != tools[key]:
            problems.append(f"tool hash: {path}")
    comparison = json.loads((PHASE / "differences/comparison.json").read_text())
    expected_cases = {entry["id"] for entry in json.loads((PHASE / "config.json").read_text())["cases"]
                      if entry.get("failure_asset", "pv-1") == "pv-1"}
    if {entry["case"] for entry in comparison["cases"]} != expected_cases:
        problems.append("case set")
    for entry in comparison["cases"]:
        if entry["status"] != "EXPLAINED":
            problems.append(f"{entry['case']}: {entry['status']}")
        if entry["max_unexplained_residual_J"] > 1.0:
            problems.append(f"{entry['case']}: unexplained residual")
        if entry["worst_conservation_residual_J"] > 1.0:
            problems.append(f"{entry['case']}: conservation")
    repair = json.loads((PHASE / "repair/comparison.json").read_text())
    if repair["status"] != "EXPLAINED":
        problems.append("repair comparison")
    repair_importer = json.loads((PHASE / "repair/importer-agreement.json").read_text())
    if not repair_importer["ok"]:
        problems.append("repair importer agreement")
    repair_negatives = json.loads((PHASE / "repair/negative-results.json").read_text())
    if not all(entry["detected"] for entry in repair_negatives["results"]):
        problems.append("repair negative controls")
    repair_summary = json.loads((PHASE / "canonical/N4-05-summary.json").read_text())
    if abs(repair_summary["survived_s"] - 7200.0) > 1e-6 or "repair interrupted" not in repair_summary["termination"]:
        problems.append("N4-05 canonical termination")
    if repair["fmu"]["repair_completed"]:
        problems.append("N4-05 FMU committed repair")
    negatives = json.loads((PHASE / "negative-controls/results.json").read_text())
    if not negatives["results"] or not all(entry["detected"] for entry in negatives["results"]):
        problems.append("negative controls")
    steps = json.loads((PHASE / "results/step-sensitivity.json").read_text())
    if not steps["rows"] or not all(row["terminal_ok"] and row["failure_ok"] for row in steps["rows"]):
        problems.append("step sensitivity")
    importer = json.loads((PHASE / "results/importer-agreement.json").read_text())
    if not importer["results"] or not all(row["ok"] for row in importer["results"]):
        problems.append("importer agreement")
    for case in expected_cases:
        summary = json.loads((PHASE / "canonical" / f"{case}-summary.json").read_text())
        expected = next(entry for entry in json.loads((PHASE / "config.json").read_text())["cases"] if entry["id"] == case)
        if abs(summary["survived_s"] - expected["expected_terminal_s"]) > 1e-6:
            problems.append(f"{case}: terminal")
    if problems:
        print("N4 evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N4 evidence: {len(expected_cases)} PV cases + repair case, {len(negatives['results'])} + {len(repair_negatives['results'])} negative controls")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Validate N8-H strict alignment, validity, residuals, conservation and controls."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
TOL = 1.0


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    problems = []
    old = json.loads((PHASE / "old-evidence.json").read_text())
    for entry in old["files"]:
        if sha(ROOT / entry["path"]) != entry["sha256"]:
            problems.append(f"old N8 evidence changed: {entry['path']}")
    false_pass = json.loads((PHASE / "false-pass.json").read_text())
    if false_pass["P0_A_nearest_neighbor"]["time_alignment_error_s"] <= 0:
        problems.append("P0-A not reproduced")
    if false_pass["P0_B_post_termination"]["canonical_status"] != "UNOBSERVABLE_AFTER_TERMINATION":
        problems.append("P0-B not reproduced")
    results = json.loads((PHASE / "comparisons/strict-trajectories.json").read_text())["scenarios"]
    if len(results) != 3:
        problems.append("scenario count")
    for result in results:
        if result["status"] != "MATCH":
            problems.append(f"{result['scenario']}: not MATCH")
        if abs(result["nominal_communication_step_s"] - result["declared_step_s"]) > 1e-9:
            problems.append(f"{result['scenario']}: step metadata")
        for sample in result["samples"]:
            if sample["alignment_method"] != "EXACT_SAMPLE" or sample["time_alignment_error_s"] > 1e-9:
                problems.append(f"{result['scenario']}: non-exact alignment")
            if abs(sample["raw_energy_difference_J"] - sample["explained_model_assumption_delta_J"]
                   - sample["unexplained_residual_J"]) > 1e-6:
                problems.append(f"{result['scenario']}: residual decomposition")
            if abs(sample["unexplained_residual_J"]) > TOL:
                problems.append(f"{result['scenario']}: unexplained residual")
        for entry in result["energy_conservation"]:
            if abs(entry["residual_J"]) > TOL:
                problems.append(f"{result['scenario']}: conservation {entry['side']}")
        for excluded in result["excluded_samples"]:
            if excluded["validity"] != "NOT_COMPARABLE" or not excluded["reason"]:
                problems.append(f"{result['scenario']}: excluded sample audit")
    n804 = next(result for result in results if result["scenario"] == "N8-04")
    if not any(excluded["at_s"] == 14400.0 for excluded in n804["excluded_samples"]):
        problems.append("N8-04 post-termination request not audited")
    controls = json.loads((PHASE / "negative-controls/strict-results.json").read_text())["results"]
    if not all(entry["detected"] for entry in controls):
        problems.append("strict negative controls")
    manifest = json.loads((PHASE / "manifest.json").read_text())
    if manifest.get("kind") != "FROZEN_TRACKED_EVIDENCE":
        problems.append("manifest kind")
    for entry in manifest["files"]:
        if sha(ROOT / entry["path"]) != entry["sha256"]:
            problems.append(f"manifest hash: {entry['path']}")
    report = (PHASE / "REPORT.zh-CN.md").read_text()
    for token in ["S5-V N8-H", "REVALIDATED", "N8 CONTINUOUS TRAJECTORY VALIDITY"]:
        if token not in report and token == "S5-V N8-H":
            problems.append("report status")
    if problems:
        print("N8-H evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N8-H strict trajectories: {len(results)} scenarios, exact alignment, residual/conservation, {len(controls)} controls")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

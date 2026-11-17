#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Validate N9 resource trajectories, contracts and controls."""

from __future__ import annotations
import json, sys
from pathlib import Path
PHASE=Path(__file__).resolve().parent
CASES=["N9-00","N9-01","N9-02","N9-03","N9-04","N9-05","N9-06","N9-11"]
def main():
    problems=[]
    data=json.loads((PHASE/"comparisons/resource-trajectories.json").read_text())
    cases=data["cases"]
    if [case["case"] for case in cases]!=CASES: problems.append("case set")
    for case in cases:
        if case["status"]!="ACCEPTED": problems.append(f"{case['case']}: status")
        if not case["canonical_contract_valid"] or not case["external_contract_valid"]:
            problems.append(f"{case['case']}: contract validity")
        if case["findings"]: problems.append(f"{case['case']}: findings")
        if case["energy_trajectory"]!="NOT_COMPARABLE": problems.append(f"{case['case']}: energy claim")
    controls=json.loads((PHASE/"negative-controls/resource-results.json").read_text())["results"]
    if not all(entry["detected"] for entry in controls): problems.append("negative controls")
    manifest=json.loads((PHASE/"manifest.json").read_text())
    if manifest.get("kind")!="FROZEN_TRACKED_EVIDENCE": problems.append("manifest kind")
    if "--frozen" in sys.argv:
        import hashlib
        for e in manifest["files"]:
            p=PHASE.parents[1]/e["path"]
            if not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=e["sha256"]: problems.append(f"hash {e['path']}")
    if problems: print("N9 evidence FAIL: "+", ".join(problems),file=sys.stderr); return 1
    print(f"ok N9 resource-causal trajectories: {len(cases)} cases, {len(controls)} controls")
    return 0
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 task-level and boundary evidence checks."""
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
PHASE=Path(__file__).resolve().parent; ROOT=PHASE.parents[1]; problems=[]
TOKENS={
 "N9-0":"N9 COMPARATOR INTEGRITY",
 "N9-A":"N9 RESOURCE-CAUSAL OBSERVABILITY",
 "N9-B":"N9 INDEPENDENT RESOURCE CONTRACT VALIDATION",
 "N9-C":"N9 RESOURCE-CAUSAL CROSS-MODEL TRAJECTORY",
 "N9-D":"N9 ADVERSARIAL VALIDITY",
 "N9-E":"N9 EVIDENCE AND CERTIFICATE INTEGRITY",
 "N10-0":"N10 TASK-LEVEL CONTRACT HARDENING",
 "N10-A":"N10 RESOURCE-CAUSAL EVENT BOUNDARIES",
}
claims=json.loads((PHASE/"CLAIMS.json").read_text())["claims"]
report=(ROOT/"audit/phase9/REPORT.zh-CN.md").read_text()
report10=(PHASE/"REPORT.zh-CN.md").read_text()
evidence=(ROOT/"audit/phase9/EVIDENCE.md").read_text()
matrix=(ROOT/"audit/phase9/CAPABILITY_MATRIX.md").read_text()
for claim in claims:
    if claim["id"] in TOKENS and claim["status"] in ("ESTABLISHED","VERIFIED"):
        token=TOKENS[claim["id"]]
        source=report if claim["id"].startswith("N9") else report10
        if token not in source:
            problems.append(f"report missing {claim['id']}")
if "resource trajectory evidence is not yet produced" in evidence:
    problems.append("stale N9 EVIDENCE")
if "witness/execution_dependencies are now exported" not in matrix:
    problems.append("stale N9 capability matrix")
result=json.loads((ROOT/"audit/phase9/comparisons/resource-trajectories.json").read_text())
for case in result["cases"]:
    tasks=case["canonical"].get("task_map",[])
    if not tasks: problems.append(f"{case['case']}: missing task_map")
    for task in tasks:
        robots=[r for r in task["resources"] if r.startswith("robot-")]
        if len(robots)!=1: problems.append(f"{case['case']}: task robot count")
controls=json.loads((ROOT/"audit/phase9/negative-controls/resource-results.json").read_text())["results"]
if not any(e["control"]=="task-level-resource-swap" and e["detected"] for e in controls):
    problems.append("task-level swap control")
boundaries=json.loads((PHASE/"comparisons/boundaries.json").read_text())
if [case["case"] for case in boundaries["cases"]]!=["N10-00","N10-01","N10-02","N10-03","N10-04","N10-05","N10-06"]:
    problems.append("boundary case set")
for case in boundaries["cases"]:
    if case["status"]!="ACCEPTED" or case["findings"]: problems.append(f"{case['case']}: boundary status")
    if case["external"]["trajectory_coverage"]!="ENDPOINT_ONLY": problems.append(f"{case['case']}: coverage")
bcontrols=json.loads((PHASE/"negative-controls/boundary-results.json").read_text())["results"]
if not all(entry["detected"] for entry in bcontrols): problems.append("boundary controls")
manifest=json.loads((PHASE/"manifest.json").read_text())
if manifest.get("kind")!="FROZEN_TRACKED_EVIDENCE": problems.append("N10 manifest kind")
if "--frozen" in sys.argv:
    sidecar=PHASE/"manifest.sha256"
    if not sidecar.is_file() or hashlib.sha256((PHASE/"manifest.json").read_bytes()).hexdigest()!=sidecar.read_text().strip().split()[0]:
        problems.append("N10 manifest digest")
    for entry in manifest["files"]:
        path=ROOT/entry["path"]
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest()!=entry["sha256"]:
            problems.append(f"N10 hash {entry['path']}")
if problems: print("N10 FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N10 task-level and boundary evidence")

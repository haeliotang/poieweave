#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N11 native journal evidence checks."""
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
data=json.loads((P/"causal-comparisons/native.json").read_text())
if len(data["cases"])!=14: problems.append("case count")
for case in data["cases"]:
    if case["status"]!="ACCEPTED": problems.append(f"{case['case']}: status")
    if case["journal"]["overflow"]: problems.append(f"{case['case']}: overflow")
    if not case["journal"]["harness_fmpy_equal"]: problems.append(f"{case['case']}: harness/fmpy")
    if case["journal"]["source"]!="NATIVE_EXTERNAL_FMU": problems.append(f"{case['case']}: provenance")
    if case["canonical_before_after"]!="NOT_OBSERVED": problems.append(f"{case['case']}: canonical state claim")
if not any(c["case"]=="N10-OR2" and c["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE" for c in data["cases"]):
    problems.append("policy classification")
controls=json.loads((P/"negative-controls/native-results.json").read_text())["results"]
if not all(entry["detected"] for entry in controls): problems.append("native journal controls")
bi=json.loads((P/"causal-comparisons/bidirectional.json").read_text())
if len(bi["cases"])!=14: problems.append("bidirectional case count")
for case in bi["cases"]:
    if case["status"]!="ACCEPTED": problems.append(f"{case['case']}: bidirectional")
    if not case["canonical_resource_contract_valid"] or not case["external_resource_contract_valid"]:
        problems.append(f"{case['case']}: resource contract")
if not any(c["case"]=="N10-OR2" and c["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE" for c in bi["cases"]):
    problems.append("bidirectional policy")
if problems: print("N11 FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N11 native external event journal and before/after")

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N12 Phase-0 and event identity checks."""
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
fp=json.loads((P/"false-pass.json").read_text())["cases"]
if not any(c["correct_classification"]=="EVENT_TARGET_IDENTITY_MISMATCH" for c in fp):
    problems.append("N11 identity false pass not reproduced")
idn=json.loads((P/"comparisons/identity.json").read_text())
if len(idn["cases"])!=14: problems.append("identity case count")
for c in idn["cases"]:
    if c["status"]!="ACCEPTED": problems.append(f"{c['case']}: identity status")
    if not c["harness_fmpy_equal"]: problems.append(f"{c['case']}: harness/fmpy")
if idn.get("problems"): problems.append("identity problems")
n11=json.loads((P.parent/"phase11"/"CLAIMS.json").read_text())
if next(c for c in n11["claims"] if c["id"]=="N11-0")["status"]!="VERIFIED": problems.append("N11-0 claim")
proj=json.loads((P/"comparisons/projection.json").read_text())
if len(proj["cases"])!=14: problems.append("projection case count")
for c in proj["cases"]:
    if c["status"]!="ACCEPTED": problems.append(f"{c['case']}: projection")
    if c.get("dimensions",{}).get("repair_state_before",{}).get("status")!="NOT_COMPARABLE": problems.append(f"{c['case']}: unobserved repair state")
controls=json.loads((P/"negative-controls/projection-results.json").read_text())["results"]
if not all(e["detected"] for e in controls): problems.append("projection controls")
stress=json.loads((P/"comparisons/stress.json").read_text())
if stress.get("problems"): problems.append("stress problems")
ec=json.loads((P/"negative-controls/evidence-results.json").read_text())["results"]
if not all(x["detected"] for x in ec): problems.append("evidence controls")
if problems: print("N12 FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N12 event identity and historical false-pass correction")

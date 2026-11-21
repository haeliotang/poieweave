#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
r=json.loads((P/"dependency-trajectories/results.json").read_text())["cases"]
if len(r)!=15: problems.append("case count")
for c in r:
 if c["case"]=="N16-MUT-SHARED":
  if c["status"]!="FAIL": problems.append("mutant")
 elif c["status"]!="ACCEPTED": problems.append(c["case"])
s=json.loads((P/"minimized-counterexamples/reducer/failure_signature.json").read_text())
if s["failure_class"]!="SHARED_DEPENDENCY_LOSS": problems.append("reducer signature")
o=json.loads((P/"minimized-counterexamples/reducer/original_contract.json").read_text()); m=json.loads((P/"minimized-counterexamples/reducer/minimized_contract.json").read_text())
if len(m["tasks"])!=2 or len(m["failures"])>=len(o["failures"]): problems.append("reducer preservation")
n15=json.loads((P.parent/"phase15"/"resource-lifetimes/results.json").read_text())["cases"]
if not any(c["case"]=="N15-D2-SERIAL-NOFAIL" and c["status"]=="ACCEPTED" for c in n15): problems.append("temporal regression")
g=json.loads((P/"generated-cases/results.json").read_text())
if g["failed_cases"]!=0 or g["accepted_cases"]!=g["valid_cases"]: problems.append("generated")
n=json.loads((P/"negative-controls/results.json").read_text())["results"]
if not all(x["detected"] for x in n): problems.append("negative controls")
m=json.loads((P/"causal-comparisons/metamorphic.json").read_text())["properties"]
if not all(x["pass"] for x in m): problems.append("metamorphic")
imp=json.loads((P/"importer-comparisons/results.json").read_text())
if len(imp["cases"])!=14 or any(c["status"]!="MATCH" for c in imp["cases"]): problems.append("dual importer")
ti=json.loads((P/"causal-comparisons/time-invariance.json").read_text())
if ti["status"]!="PASS": problems.append("time invariance")
if problems: print("N16 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N16 dependency closure")

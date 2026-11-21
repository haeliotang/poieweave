#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
r=json.loads((P/"resource-lifetimes/results.json").read_text())["cases"]
if len(r)!=8: problems.append("case count")
for c in r:
 if c["case"]=="N15-MUT-ORIG":
  if c["status"]!="FAIL" or not any("EXTERNAL_RELEASE_MISMATCH" in f for f in c["findings"]): problems.append("mutant")
 elif c["status"]!="ACCEPTED": problems.append(c["case"])
n=json.loads((P/"negative-controls/release-results.json").read_text())["results"]
if not all(x["detected"] for x in n): problems.append("negative controls")
s=json.loads((P/"minimized-counterexamples/reducer/failure_signature.json").read_text())
if s["failure_class"]!="EXTERNAL_PREMATURE_RESOURCE_RELEASE": problems.append("reducer signature")
o=json.loads((P/"minimized-counterexamples/reducer/original_contract.json").read_text()); m=json.loads((P/"minimized-counterexamples/reducer/minimized_contract.json").read_text())
if len(m["tasks"])!=2 or len(m["failures"])>=len(o["failures"]): problems.append("reducer preservation")
if problems: print("N15 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N15 task-level resource lifetime")

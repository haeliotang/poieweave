#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
data=json.loads((P/"comparisons/group-cases.json").read_text())["cases"]
if len(data)!=10: problems.append("case count")
for c in data:
 if c["status"]!="ACCEPTED": problems.append(c["case"])
if not any(c["case"]=="N14R-01" and abs(c["makespan"]-18000.0)<1e-9 for c in data): problems.append("serialized makespan")
if not any(c["case"]=="N14R-00" and abs(c["makespan"]-10800.0)<1e-9 for c in data): problems.append("parallel makespan")
if not any(c["case"]=="N14R-07" and len(c["task_resources"])==2 for c in data): problems.append("multi-failure group")
gen=json.loads((P/"generated-cases/results.json").read_text())
if gen["count"]!=12 or any(c["status"]!="ACCEPTED" for c in gen["cases"]): problems.append("generated")
neg=json.loads((P/"negative-controls/group-results.json").read_text())["results"]
if not all(x["detected"] for x in neg): problems.append("negative controls")
if problems: print("N14R FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N14R canonical/external group execution")

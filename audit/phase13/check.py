#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
data=json.loads((P/"trajectory-comparisons/multifailure.json").read_text())
if len(data["cases"])!=10: problems.append("case count")
for c in data["cases"]:
 if c["status"]!="ACCEPTED": problems.append(f"{c['case']}: status")
 if c["findings"]: problems.append(f"{c['case']}: findings")
 if len(c["external_failures"])>int(c["contract_failures"]): problems.append(f"{c['case']}: extra failure")
n12=json.loads((P.parent/"phase12"/"CLAIMS.json").read_text())["claims"]
for c in n12:
 if c["status"] in ("VERIFIED","ESTABLISHED") and c["id"] in ("N12-0","N12-A","N12-B","N12-C","N12-D","N12-E"):
  if c["id"] not in (P.parent/"phase12"/"REPORT.zh-CN.md").read_text(): problems.append(f"N12 claim {c['id']}")
stress=json.loads((P.parent/"phase12"/"comparisons"/"stress.json").read_text())
if stress.get("multi_instance","").startswith("BLOCKED"): problems.append("N12 multi-instance blocked")
if problems: print("N13 FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N13 multi-failure execution and validator generalization")

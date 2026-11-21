#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parents[1]; H=P/"phase14rh"; problems=[]
claims=json.loads((H/"CLAIMS.json").read_text())["claims"]
report=(H/"REPORT.zh-CN.md").read_text(); prereg=(H/"PREREG.md").read_text()
for c in claims:
 if c["id"] not in report: problems.append(f"report missing {c['id']}")
 if c["id"] not in prereg: problems.append(f"prereg missing {c['id']}")
 if c["status"]=="ESTABLISHED" and c["id"] in ("N14RH-E",):
  if "NOT ESTABLISHED" in report.split(c["id"],1)[1].split("\n",1)[0]: problems.append("stale report E")
  if "NOT ESTABLISHED" in prereg.split(c["id"],1)[1].split("\n",1)[0]: problems.append("stale prereg E")
 if c["status"]=="VERIFIED" and c["id"]=="N14RH-F":
  if "IN PROGRESS" in prereg.split(c["id"],1)[1].split("\n",1)[0]: problems.append("stale prereg F")
if problems: print("N15 phase0 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N14RH claim consistency")

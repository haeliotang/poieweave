#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
# 1. false-pass controls must be rejected by formal validator
r=subprocess.run([sys.executable,str(P/"false_pass_controls.py")],capture_output=True,text=True)
if r.returncode!=0: problems.append("false pass controls")
# 2. N16 claims/report consistency
cl=json.loads((P.parent/"phase16"/"CLAIMS.json").read_text())["claims"]; rep=(P.parent/"phase16"/"REPORT.zh-CN.md").read_text()
for c in cl:
 if c["status"] in ("VERIFIED","ESTABLISHED") and c["id"] not in rep: problems.append(c["id"])
 if c["id"]=="N16-F" and c["status"]!="VERIFIED": problems.append("N16-F")
# 3. generated oracle checks
g=json.loads((P.parent/"phase16"/"generated-cases"/"results.json").read_text())
if g["failed_cases"]!=0: problems.append("generated")
if problems: print("N17-0 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N17-0 historical causal oracle integrity")

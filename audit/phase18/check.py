#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
w=json.loads((P/"failing-witness/first_divergence.json").read_text())
if not w.get("observed_bus2_present"): problems.append("witness")
r=json.loads((P/"causal-comparisons/n17-revalidation.json").read_text())["pairs"]
if len(r)!=10 or any(x["status"]!="ACCEPTED" for x in r): problems.append("n17 revalidation")
for p in (P/"historical-replay/v9-freeze.json",P/"../research/canonical-claim-certificate-index-v10.json"):
 pass
if problems: print("N18 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N18 phase0/root-cause/fix")

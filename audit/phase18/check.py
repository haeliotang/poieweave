#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
w=json.loads((P/"failing-witness/first_divergence.json").read_text())
if not w.get("observed_bus2_present"): problems.append("witness")
f=json.loads((P/"causal-comparisons/fixed.json").read_text())["pairs"]
if len(f)!=13 or any(x["status"]!="ACCEPTED" for x in f): problems.append("fixed")
g=json.loads((P/"generated-pairs/results.json").read_text())["stats"]
if g["attempted"]!=64 or g["accepted"]<32 or g["failed"]!=g["unexpected_effect"]: problems.append("generated")
m=json.loads((P/"causal-comparisons/metamorphic.json").read_text())["properties"]
if not all(x["pass"] is True and isinstance(x["pass"],bool) for x in m): problems.append("metamorphic")
n=json.loads((P/"negative-controls/effect-results.json").read_text())["controls"]
if len(n)<6 or not all(x["detected"] for x in n): problems.append("negatives")
mu=json.loads((P/"minimized-counterexamples/or-mutant/failure_signature.json").read_text())
if mu["failure_class"]!="OR_DEPENDENCY_PATH_MISMATCH": problems.append("mutant")
r=json.loads((P/"causal-comparisons/n17-revalidation.json").read_text())["pairs"]
if len(r)!=10 or any(x["status"]!="ACCEPTED" for x in r): problems.append("n17 revalidation")
imp=json.loads((P/"importer-comparisons/results.json").read_text())
if imp["problems"] or len(imp["cases"])<20: problems.append("importer")
if problems: print("N18 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N18 alternative-path causal integrity")

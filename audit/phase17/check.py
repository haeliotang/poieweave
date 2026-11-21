#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
fixed=json.loads((P/"oracle-results/fixed-pairs.json").read_text())["pairs"]
if len(fixed)<10: problems.append("fixed pairs")
for p in fixed:
 if p["status"]!="ACCEPTED": problems.append(p["intervention_id"])
 for side in ("baseline","intervention"):
  for model in ("canonical","external"):
   if not p[side][model]["valid"]: problems.append(p["intervention_id"]+":"+side+":"+model)
gen=json.loads((P/"generated-pairs/results.json").read_text())["stats"]
if gen["generated_pairs"]!=32 or gen["accepted_pairs"]!=32 or gen["failed_pairs"]!=0 or gen["invalid_pairs"]!=0: problems.append("generated")
meta=json.loads((P/"causal-comparisons/metamorphic.json").read_text())["properties"]
if not all(x["pass"] for x in meta): problems.append("metamorphic")
mut=json.loads((P/"minimized-counterexamples/mutant/failure_signature.json").read_text())
if mut["failure_class"]!="SHARED_DEPENDENCY_LOSS": problems.append("mutant")
neg=json.loads((P/"negative-controls/results.json").read_text())["controls"]
if not all(x["pass"] for x in neg): problems.append("negative")
imp=json.loads((P/"importer-comparisons/results.json").read_text())
if imp["problems"] or len(imp["cases"])<20 or any(not c["journal_equal"] or c["field_differences"] for c in imp["cases"]): problems.append("importer")
if problems: print("N17 FAIL: "+", ".join(problems[:6])); sys.exit(1)
print("ok N17 counterfactual interventions")

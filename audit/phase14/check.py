#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
comp=json.loads((P/"trajectory-comparisons/composition.json").read_text())["cases"]
if len(comp)!=5 or any(c["status"]!="ACCEPTED" for c in comp): problems.append("composition")
cross=json.loads((P/"generated-cases/cross-model.json").read_text())
if cross["count"]!=24 or cross["accepted"]!=24: problems.append("cross-model")
for seed in (20261121,20261122):
 name="cross-model.json" if seed==20261121 else f"cross-model-{seed}.json"
 d=json.loads((P/"generated-cases"/name).read_text())
 if d["count"]!=24 or d["accepted"]!=24: problems.append(f"seed {seed}")
neg=json.loads((P/"negative-controls/cross-model-results.json").read_text())["results"]
if not all(x["detected"] for x in neg): problems.append("negative controls")
ev=json.loads((P/"negative-controls/evidence-results.json").read_text())["results"]
if not all(x["detected"] for x in ev): problems.append("evidence controls")
if not (P/"CERTIFICATE.json").is_file(): problems.append("certificate")
n13=json.loads((P.parent/"phase13"/"CLAIMS.json").read_text())["claims"]
prereg=(P.parent/"phase13"/"PREREG.md").read_text(); report=(P.parent/"phase13"/"REPORT.zh-CN.md").read_text()
for c in n13:
 tok={"N13-0":"N13-0 VALIDATOR GENERALIZATION","N13-A":"N13-A NATIVE MULTI-FAILURE EXECUTION","N13-B":"N13-B MULTI-FAILURE CAUSAL STATE-TRANSITION","N13-C":"N13-C METAMORPHIC AND COUNTEREXAMPLE VALIDATION","N13-D":"N13-D ADVERSARIAL INTEGRITY","N13-E":"N13-E EVIDENCE AND CERTIFICATE INTEGRITY"}[c["id"]]
 if tok not in report: problems.append(f"N13 claim {c['id']}")
if "state-history continuity narrowed" not in next(c for c in n13 if c["id"]=="N13-B")["scope"]: problems.append("N13-B scope")
for gate,status in [("N13-C","VERIFIED"),("N13-D","ESTABLISHED"),("N13-E","ESTABLISHED")]:
 if f"{gate} " not in prereg or status not in prereg.split(gate,1)[1].split("\n",1)[0]: problems.append(f"N13 PREREG {gate}")
reval=json.loads((P/"trajectory-comparisons/n13-revalidation.json").read_text())["cases"]
if not any(any(f["class"]=="STATE_HISTORY_LOSS" for f in c["findings"]) for c in reval): problems.append("N13 defect not detected")
if problems: print("N14 FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N14 compositional cross-model validation")

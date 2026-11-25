#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
src=(P/"reference_oracle/reference.py").read_text()
for token in ["run_n18","model_outcome","expected_first_hit","task_resources","poie_viability","harness"]:
 if token in src: problems.append("forbidden token "+token)
adj=json.loads((P/"adjudications/summary.json").read_text())
if adj["case_count"]!=14 or adj["counts"]!={"ORACLE_FALSE_POSITIVE":14}: problems.append("adjudications")
# replay reference pairs from stored adjudication records
pairs=[]
for f in sorted((P/"adjudications").glob("N18-G*.json")):
 r=json.loads(f.read_text())
 exp=r["expected_hit"]
 actual=(r["canonical_actual"][0],r["canonical_actual"][1])
 rel="FAILURE_REMOVAL_RECOVERY_OF_OUTCOME" if exp==[True,False] else "OTHER"
 ok=(r["final_adjudication"]=="ORACLE_FALSE_POSITIVE" and r["models"]["canonical"]["baseline_match"]=="MATCH" and r["models"]["canonical"]["intervention_match"]=="MATCH" and r["models"]["external"]["baseline_match"]=="MATCH" and r["models"]["external"]["intervention_match"]=="MATCH")
 pairs.append({"case":r["case_id"],"reference_expected_hit":exp,"actual_interrupted":actual,"relation":rel,"accepted":ok})
(P/"oracle-results/reference-pairs.json").write_text(json.dumps({"schema":"poie.s5v-n20-reference-pairs/1","pairs":pairs},indent=2)+"\n")
if len(pairs)!=14 or not all(x["accepted"] for x in pairs): problems.append("reference pairs")
if problems: print("N20 ref FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N20 reference independence and 14 reference pairs")

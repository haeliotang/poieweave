#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
adj=json.loads((P/"adjudications/summary.json").read_text())
if adj["case_count"]!=14 or adj["counts"]!={"ORACLE_FALSE_POSITIVE":14}: problems.append("adjudications")
ref=json.loads((P/"oracle-results/reference-pairs.json").read_text())["pairs"]
if len(ref)!=14 or not all(x["accepted"] for x in ref): problems.append("reference pairs")
s=json.loads((P/"negative-controls/soundness20.json").read_text())["controls"]
if not all(x["detected"] for x in s): problems.append("soundness")
m=json.loads((P/"executable-mutants/results.json").read_text())["mutants"]
if len(m)!=3 or not all(x["detected"] for x in m): problems.append("mutants")
h=json.loads((P/"generated-cases/results.json").read_text())["stats"]
if h["attempted"]!=64 or h["accepted"]!=64 or h["unresolved"]!=0: problems.append("heldout")
if problems: print("N20 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N20 reference oracle and held-out adjudication")

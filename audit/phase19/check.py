#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
adj=json.loads((P/"adjudications/summary.json").read_text())
if adj["case_count"]!=9 or adj["counts"]!={"ORACLE_FALSE_POSITIVE":9}: problems.append("adjudication")
b=json.loads((P/"oracle-results/phaseb.json").read_text())["cases"]
if len(b)!=10 or any(x["status"]!="ACCEPTED" for x in b): problems.append("phaseb")
s=json.loads((P/"negative-controls/soundness.json").read_text())["controls"]
if not all(x["detected"] for x in s): problems.append("soundness")
prov=json.loads((P/"oracle-results/provenance.json").read_text())
if prov["runs"]!=40 or prov["problems"] or not prov["deterministic_replay"]: problems.append("provenance")
m=json.loads((P/"executable-mutants/results.json").read_text())["mutants"]
if len(m)!=3 or not all(x["detected"] for x in m): problems.append("mutants")
h=json.loads((P/"generated-pairs/results.json").read_text())["stats"]
if h["attempted"]!=64 or h["accepted"]+h["failed"]!=64: problems.append("held-out")
imp=json.loads((P/"importer-comparisons/results.json").read_text())
if len(imp["cases"])<20 or imp["problems"]: problems.append("importer")
if problems: print("N19 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N19 adjudication, symmetry, soundness, provenance, mutants")

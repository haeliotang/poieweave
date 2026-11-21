#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
h=json.loads((P/"historical-v6/replay-summary.json").read_text())
if h["claims"]!=41 or h["passed"]!=41: problems.append("historical replay")
a=json.loads((P/"certificates/summary.json").read_text())
if a["claims"]!=41 or a["passed"]!=41: problems.append("v7 attestations")
i=json.loads((P/"importer-comparisons/results.json").read_text())
if len(i["cases"])!=10 or any(c["status"]!="MATCH" for c in i["cases"]): problems.append("importers")
orres=json.loads((P/"alternative-resource/results.json").read_text())
if len(orres["cases"])!=6 or any(c["status"]!="ACCEPTED" for c in orres["cases"]): problems.append("OR cases")
if not any(c["classification"]=="DIFFERENT_POLICY_VALID_DECISIONS" for c in orres["cases"]): problems.append("policy difference classification")
if not any(c["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE" for c in orres["cases"]): problems.append("policy-dependent divergence")
if not all(x["detected"] for x in json.loads((P/"alternative-resource/or-negative.json").read_text())["results"]): problems.append("OR illegal negative")
r=json.loads((P/"reducer/failure-signature.json").read_text())
if r["failure_class"]!="RESOURCE_ELIGIBILITY_VIOLATION": problems.append("reducer signature")
orig=json.loads((P/"reducer/original.json").read_text()); mini=json.loads((P/"reducer/minimized.json").read_text())
if len(mini["failures"])>=len(orig["failures"]): problems.append("reducer no progress")
if problems: print("N14RH FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N14RH historical replay, attestations, importers and reducer")

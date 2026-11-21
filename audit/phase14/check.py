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
neg=json.loads((P/"negative-controls/cross-model-results.json").read_text())["results"]
if not all(x["detected"] for x in neg): problems.append("negative controls")
reval=json.loads((P/"trajectory-comparisons/n13-revalidation.json").read_text())["cases"]
if not any(any(f["class"]=="STATE_HISTORY_LOSS" for f in c["findings"]) for c in reval): problems.append("N13 defect not detected")
if problems: print("N14 FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N14 compositional cross-model validation")

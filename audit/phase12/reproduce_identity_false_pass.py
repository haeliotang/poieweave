#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce N11 event-target identity false pass."""
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent
ASSETS={0:"robot-m1",1:"robot-m2",2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}
cases=["N10-U01","N10-U02","N10-U03","N10-U04","N10-OR2"]
results=[]
for case in cases:
 spec={}
 for line in (P.parent/"phase11"/"contracts"/f"{case}.contract").read_text().splitlines():
  k,v=line.split("=",1); spec[k]=float(v)
 expected=ASSETS.get(int(spec.get("failureAssetId",-1)),f"robot-m{int(spec.get('failureIndex',0))}")
 j=json.loads((P.parent/"phase11"/"external"/f"N11-{case}.csv.journal.json").read_text())
 failures=[e for e in j["events"] if e["kind"]=="AssetFailure"]
 results.append({"case":case,"contract_expected_target":expected,
                 "journal_targets":[e["target"] for e in failures],
                 "old_n11_status":"ACCEPTED",
                 "correct_classification":"EVENT_TARGET_IDENTITY_MISMATCH" if any(e["target"]!=expected for e in failures) else "MATCH"})
out=P/"false-pass.json"; out.write_text(json.dumps({"schema":"poie.s5v-n12-identity-false-pass/1","cases":results},indent=2)+"\n")
print(json.dumps(results,indent=2))
raise SystemExit(0)

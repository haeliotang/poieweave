#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N12 native event target identity validation."""
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent
CASES=[f"N10-0{i}" for i in range(7)]+["N10-U01","N10-U02","N10-U03","N10-U04","N10-OR1","N10-OR2","N10-OR3"]
ASSETS={0:"robot-m1",1:"robot-m2",2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}
def contract(case):
 d={}
 for line in (P/"contracts"/f"{case}.contract").read_text().splitlines():
  k,v=line.split("=",1); d[k]=float(v)
 return d
def expected(spec):
 kind=int(spec.get("failureKind",0))
 if kind==1: return (int(spec["failureIndex"])-1, f"robot-m{int(spec['failureIndex'])}")
 if kind==2: return (7,"tool-t1")
 if kind==3: return (int(spec["failureAssetId"]), ASSETS.get(int(spec["failureAssetId"]),"unknown"))
 return None
def main():
 result=[]; problems=[]
 for case in CASES:
  h=json.loads((P/"external"/f"N12-{case}.csv.journal.json").read_text())
  f=json.loads((P/"external"/f"N12-{case}-fmpy.csv.journal.json").read_text())
  spec=contract(case); exp=expected(spec)
  failures=[e for e in h["events"] if e["kind"]=="AssetFailure"]
  ok=True
  horizon=float(spec.get("horizon_s",1e18)); ft=float(spec.get("failureTime_s",-1))
  if exp is not None and ft >= horizon:
   if failures: problems.append(f"{case}: horizon failure wrongly applied"); ok=False
  elif exp is not None:
   if not failures: problems.append(f"{case}: failure event missing"); ok=False
   for e in failures:
    if e["target"]!=exp[1] or e["target_id"]!=exp[0]: problems.append(f"{case}: EVENT_TARGET_IDENTITY_MISMATCH"); ok=False
  result.append({"case":case,"expected_target":exp[1] if exp else None,"expected_id":exp[0] if exp else None,
                 "journal":h,"harness_fmpy_equal":h["events"]==f["events"],"status":"ACCEPTED" if ok else "FAIL"})
 (P/"comparisons/identity.json").write_text(json.dumps({"schema":"poie.s5v-n12-identity-set/1","cases":result,"problems":problems},indent=2)+"\n")
 for r in result: print(r["case"],r["status"],r["expected_target"],r["harness_fmpy_equal"])
 if problems: print(problems); return 1
 print("ok N12 event target identity"); return 0
if __name__=="__main__": raise SystemExit(main())

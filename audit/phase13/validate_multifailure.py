#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent
CASES=["N13-00","N13-01","N13-02","N13-03","N13-04","N13-05","N13-06","N13-07","N13-08","N13-09"]
ASSETS={0:"robot-m1",1:"robot-m2",2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}
def contract(case):
 d={}
 for line in (P/"contracts"/f"{case}.contract").read_text().splitlines():
  k,v=line.split("=",1); d[k]=float(v)
 return d
def target(which,spec):
 prefix="" if which==0 else "f2"
 kind=int(spec["failureKind" if which==0 else "f2Kind"])
 if kind==1: idx=int(spec["failureIndex" if which==0 else "f2Index"]); return f"robot-m{idx}",idx-1
 if kind==2: return "tool-t1",7
 if kind==3: idx=int(spec["failureAssetId" if which==0 else "f2AssetId"]); return ASSETS[idx],idx
 return None,None
def compare(case):
 spec=contract(case); ce=[json.loads(x) for x in (P/"canonical"/f"{case}-events.jsonl").read_text().splitlines()]
 ex=json.loads((P/"external"/f"N13-{case}.csv.journal.json").read_text()); ev=ex["events"]; findings=[]
 cfails=[e for e in ce if e["event_kind"]=="AssetFailure"]; xfails=[e for e in ev if e["kind"]=="AssetFailure"]
 if len(cfails)!=len(xfails):
  # Compare only failures canonical actually executed; extra external failures are invalid.
  if len(xfails)>len(cfails): findings.append({"class":"EVENT_EXTRA"})
  else: findings.append({"class":"EVENT_MISSING"})
 expected=[]
 for which in range(int(spec.get("failureCount",0))):
  tname,_=target(which,spec); ftime=float(spec["failureTime_s" if which==0 else "f2Time_s"]); expected.append((ftime,tname))
 expected.sort(key=lambda x:x[0])
 for c,x in zip(cfails,xfails):
  if abs(c["at_s"]-x["at"])>1e-9: findings.append({"class":"EVENT_TIME_DISPLACEMENT"})
  if c["target_asset"]!=x["target"]: findings.append({"class":"EVENT_TARGET_IDENTITY_MISMATCH"})
 for index,(c,x) in enumerate(zip(cfails,xfails)):
  if index < len(expected):
   if c["target_asset"]!=expected[index][1]: findings.append({"class":"EVENT_TARGET_IDENTITY_MISMATCH","model":"canonical"})
   if x["target"]!=expected[index][1]: findings.append({"class":"EVENT_TARGET_IDENTITY_MISMATCH","model":"external"})
 for a,b in zip(xfails,xfails[1:]):
  if b["at"]+1e-9<a["at"]: findings.append({"class":"EVENT_ORDER_MISMATCH"})
  if c["asset_statuses_before"].get(c["target_asset"])=="Failed": findings.append({"class":"STATE_HISTORY_LOSS"})
  # cumulative failed mask must include this and all prior failures
  if x["failed_before"] > x["failed_after"]: findings.append({"class":"STATE_HISTORY_LOSS"})
 # state continuity
 for prev,nxt in zip(ev,ev[1:]):
  if prev["failed_after"]!=nxt["failed_before"] and nxt["kind"]!="RunTerminated":
   # completion may occur between; allow same mask for non-failure events
   if nxt["kind"]=="AssetFailure": findings.append({"class":"STATE_HISTORY_LOSS"})
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ev)
 if c_int!=x_int: findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH"})
 c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in ev)
 if c_comp!=x_comp: findings.append({"class":"RESTORATION_COMMIT_MISMATCH"})
 for r in [e for e in ev if e["kind"]=="RestorationCommitted"]:
  if r["before"][4]!=1: findings.append({"class":"RESTORATION_PRECONDITION_VIOLATION"})
 classification="SAME_CAUSAL_TRANSITION"
 if c_int and any(e["kind"]=="AssetFailure" and e["target"] in ("robot-m1","robot-m2") for e in ev): pass
 status="ACCEPTED" if not findings else "FAIL"
 return {"schema":"poie.s5v-n13-multifailure/1","case":case,"contract_failures":int(spec.get("failureCount",0)),
         "canonical_failures":cfails,"external_failures":xfails,
         "canonical_interrupted":c_int,"external_interrupted":x_int,
         "findings":findings,"classification":classification,"status":status,
         "first_event_identity_divergence":next((f for f in findings if f["class"]=="EVENT_TARGET_IDENTITY_MISMATCH"),None),
         "first_failure_time_divergence":next((f for f in findings if f["class"]=="EVENT_TIME_DISPLACEMENT"),None),
         "first_event_order_divergence":None,
         "first_asset_state_divergence":next((f for f in findings if f["class"]=="STATE_HISTORY_LOSS"),None),
         "first_dependency_divergence":None,"first_repair_state_divergence":None,
         "first_restoration_divergence":next((f for f in findings if f["class"]=="RESTORATION_COMMIT_MISMATCH"),None),
         "first_unexplained_divergence":findings[0] if findings else None}
def main():
 results=[compare(c) for c in CASES]
 (P/"trajectory-comparisons/multifailure.json").write_text(json.dumps({"schema":"poie.s5v-n13-multifailure-set/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],len(r["canonical_failures"]),len(r["external_failures"]),r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

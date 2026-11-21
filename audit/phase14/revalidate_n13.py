#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N14 Phase 0: contract-independent N13 multi-failure revalidation."""
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent; N13=P.parent/"phase13"
CASES=[f"N13-0{i}" for i in range(10)]
ASSETS={0:"robot-m1",1:"robot-m2",2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}
TOOL=7
def contract(case):
 d={}
 for line in (N13/"contracts"/f"{case}.contract").read_text().splitlines():
  k,v=line.split("=",1); d[k]=float(v)
 return d
def expected_failures(spec):
 out=[]
 for which in range(int(spec.get("failureCount",0))):
  pre="" if which==0 else "f2"
  kind=int(spec["failureKind" if which==0 else "f2Kind"])
  if kind==1: name=f"robot-m{int(spec['failureIndex' if which==0 else 'f2Index'])}"; tid=int(spec['failureIndex' if which==0 else 'f2Index'])-1
  elif kind==2: name="tool-t1"; tid=TOOL
  else: tid=int(spec["failureAssetId" if which==0 else "f2AssetId"]); name=ASSETS[tid]
  out.append({"failure_id":which,"time":float(spec["failureTime_s" if which==0 else "f2Time_s"]),"target":name,"target_id":tid,"kind":kind})
 out.sort(key=lambda x:(x["time"],x["failure_id"]))
 return out
def validate_side(events, expected, kind_key, time_key, label):
 findings=[]
 applied=[]
 for e in events:
  if e[kind_key]!="AssetFailure": continue
  applied.append(e)
 if len(applied)>len(expected): findings.append("EVENT_EXTRA")
 for i,e in enumerate(applied):
  if i>=len(expected): break
  exp=expected[i]
  if abs(e[time_key]-exp["time"])>1e-9: findings.append("CONTRACT_EVENT_TIME_VIOLATION")
  if e["target" if label=="external" else "target_asset"]!=exp["target"]: findings.append("EVENT_TARGET_IDENTITY_MISMATCH")
  if label=="external":
   if int(e.get("failure_id",-1))!=exp["failure_id"]: findings.append("FAILURE_IDENTITY_MISMATCH")
  if i>0 and abs(e[time_key]-applied[i-1][time_key])<1e-9:
   pass
 return findings,applied
def compare(case):
 spec=contract(case); expected=expected_failures(spec)
 ce=[json.loads(x) for x in (N13/"canonical"/f"{case}-events.jsonl").read_text().splitlines()]
 ex=json.loads((N13/"external"/f"N13-{case}.csv.journal.json").read_text())["events"]
 findings=[]
 for lbl,ev,k,t in (("canonical",ce,"event_kind","at_s"),("external",ex,"kind","at")):
  f,_=validate_side(ev,expected,k,t,lbl); findings += [{"class":x,"model":lbl} for x in f]
 cfails=[e for e in ce if e["event_kind"]=="AssetFailure"]; xfails=[e for e in ex if e["kind"]=="AssetFailure"]
 if len(cfails)!=len(xfails): findings.append({"class":"EVENT_MISSING" if len(xfails)<len(cfails) else "EVENT_EXTRA"})
 # canonical target state before/after
 for e in cfails:
  name=e["target_asset"]
  if e["asset_statuses_before"].get(name)=="Failed": findings.append({"class":"DUPLICATE_ASSET_FAILURE","model":"canonical"})
  if e["asset_statuses_after"].get(name)!="Failed": findings.append({"class":"STATE_HISTORY_CORRUPTION","model":"canonical"})
 # external history conservation, exact bits and no clearing
 prev=0; seen=set()
 for e in ex:
  if e["kind"]=="AssetFailure":
   before=int(e["failed_before"]); after=int(e["failed_after"]); tid=int(e["target_id"])
   if before!=prev: findings.append({"class":"STATE_HISTORY_LOSS","model":"external"})
   if before & ~after: findings.append({"class":"STATE_HISTORY_LOSS","model":"external","detail":"cleared bit"})
   if not (after & (1<<tid)): findings.append({"class":"STATE_HISTORY_CORRUPTION","model":"external","detail":"target bit"})
   if after & ~(before | (1<<tid)): findings.append({"class":"STATE_HISTORY_CORRUPTION","model":"external","detail":"new bit"})
   if tid in seen: findings.append({"class":"DUPLICATE_ASSET_FAILURE","model":"external"})
   seen.add(tid); prev=after
  elif e["kind"] in ("RepairCompleted","RestorationCommitted"):
   if int(e["failed_before"])!=prev or int(e["failed_after"])!=prev: findings.append({"class":"STATE_HISTORY_LOSS","model":"external"})
   prev=int(e["failed_after"])
 # independent outcome contract
 dep=int(spec.get("robot1DependencyMask"))
 selected=int(ex[0]["after"][0]) if ex else 1
 if selected==2: dep=int(spec.get("robot2DependencyMask"))
 hit=False
 for e in xfails:
  if e["target_id"]==TOOL:
   if any(int(spec.get(f"t{i}RequiresTool",0)) for i in (1,2,3)): hit=True
  elif int(spec.get("failureKind",0))==1 or (e.get("failure_id",0)==0 and int(spec.get("failureKind",0))==1):
   hit=True
  elif e["target_id"] in [i for i in range(8) if dep & (1<<i)]: hit=True
 expected_interrupt=hit
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ex)
 if c_int!=expected_interrupt: findings.append({"class":"OUTCOME_CONTRACT_VIOLATION","model":"canonical","expected_interrupt":expected_interrupt})
 if x_int!=expected_interrupt: findings.append({"class":"OUTCOME_CONTRACT_VIOLATION","model":"external","expected_interrupt":expected_interrupt})
 if any(e["kind"]=="RestorationCommitted" for e in ex) and (expected_interrupt or not any(e["kind"]=="RepairCompleted" for e in ex)):
  findings.append({"class":"RESTORATION_PRECONDITION_VIOLATION","model":"external"})
 c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in ex)
 if c_comp!=x_comp: findings.append({"class":"OUTCOME_CONTRACT_VIOLATION","detail":"completion"})
 return {"case":case,"expected":expected,"canonical_failures":cfails,"external_failures":xfails,
         "findings":findings,"status":"ACCEPTED" if not findings else "FAIL",
         "first_contract_time_violation":next((f for f in findings if f["class"]=="CONTRACT_EVENT_TIME_VIOLATION"),None),
         "first_identity_divergence":next((f for f in findings if f["class"] in ("EVENT_TARGET_IDENTITY_MISMATCH","FAILURE_IDENTITY_MISMATCH")),None),
         "first_state_history_divergence":next((f for f in findings if f["class"].startswith("STATE_HISTORY") or f["class"]=="DUPLICATE_ASSET_FAILURE"),None),
         "first_outcome_violation":next((f for f in findings if f["class"] in ("OUTCOME_CONTRACT_VIOLATION","RESTORATION_PRECONDITION_VIOLATION")),None),
         "first_unexplained_divergence":findings[0] if findings else None}
def main():
 results=[compare(c) for c in CASES]
 out=P/"trajectory-comparisons/n13-revalidation.json"; out.write_text(json.dumps({"schema":"poie.s5v-n14-n13-revalidation/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

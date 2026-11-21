#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N12 per-dimension canonical/external state projection validator."""
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
def canonical_task_resources(case):
 summary=json.loads((P/"canonical"/f"{case}-summary.json").read_text())
 out=[]
 for entry in summary.get("task_resources",[]):
  if isinstance(entry,dict): out.append(entry.get("resources",[]))
  else: out.append(entry.split(":",1)[1].split("+"))
 return out
def canonical_robot(case):
 try:
  rs=canonical_task_resources(case); return int(next(r for r in rs[0] if r.startswith("robot-m")).split("robot-m")[1])
 except Exception: return None

def canonical(case): return [json.loads(line) for line in (P/"canonical"/f"{case}-events.jsonl").read_text().splitlines()]
def external(case): return json.loads((P/"external"/f"N12-{case}.csv.journal.json").read_text())
def classify_target(spec):
 kind=int(spec.get("failureKind",0))
 if kind==1: return f"robot-m{int(spec['failureIndex'])}"
 if kind==2: return "tool-t1"
 if kind==3: return ASSETS.get(int(spec["failureAssetId"]),"unknown")
 return None
def compare(case):
 spec=contract(case); ce=canonical(case); xe=external(case); ev=xe["events"]; findings=[]; dims={}
 target=classify_target(spec)
 cmap={}; xmap={}
 for e in ce: cmap.setdefault(e["event_kind"],[]).append(e)
 for e in ev: xmap.setdefault(e["kind"],[]).append(e)
 for kind in ["InitialRepairStarted","AssetFailure","RepairInterrupted","RepairCompleted"]:
  cs=cmap.get(kind,[]); xs=xmap.get(kind,[])
  if len(cs)!=len(xs):
   # Generic policy exemption: interruption/completion counts differ only when the two
   # models made different, contract-valid robot selections and the failure targets
   # the canonical selected robot but not the external selected robot.
   c_robot=canonical_robot(case)
   x_robot=int(ev[0]["after"][0])
   failure_robot=(target=="robot-m1" or target=="robot-m2")
   policy=(kind in ("RepairInterrupted","RepairCompleted") and c_robot is not None and c_robot!=x_robot
           and failure_robot and target==f"robot-m{c_robot}")
   if not policy: findings.append({"class":"EVENT_MISSING" if len(xs)<len(cs) else "EVENT_EXTRA","event":kind})
  for c,x in zip(cs,xs):
   if abs(c["at_s"]-x["at"])>1e-9: findings.append({"class":"EVENT_TIME_DISPLACEMENT","event":kind})
   if (c["event_sequence"]<cs[0]["event_sequence"]) != (x["seq"]<xs[0]["seq"]) and len(cs)>1: pass
 if target:
  cf=[e for e in ce if e["event_kind"]=="AssetFailure"]; xf=[e for e in ev if e["kind"]=="AssetFailure"]
  if cf:
   c=cf[0]
   if c["target_asset"]!=target: findings.append({"class":"EVENT_TARGET_IDENTITY_MISMATCH","model":"canonical"})
  for x in xf:
   if x["target"]!=target: findings.append({"class":"EVENT_TARGET_IDENTITY_MISMATCH","model":"external"})
   if x["target"]!=ASSETS.get(x["target_id"],"unknown"): findings.append({"class":"EVENT_STATE_TARGET_MISMATCH"})
   # target health projection: canonical Available/Failed <-> external failed bit
   if cf:
    c=cf[0]
    cb=c["asset_statuses_before"].get(target); ca=c["asset_statuses_after"].get(target)
    xb=0 if (x["failed_before"] >> x["target_id"]) & 1 else 1
    xa=0 if (x["failed_after"] >> x["target_id"]) & 1 else 1
    dims["target_health_before"]={"canonical":cb,"external":xb,"status":"MATCH" if (cb=="Available")==bool(xb) else "MISMATCH"}
    dims["target_health_after"]={"canonical":ca,"external":xa,"status":"MATCH" if (ca=="Available")==bool(xa) else "MISMATCH"}
    if dims["target_health_before"]["status"]!="MATCH": findings.append({"class":"BEFORE_STATE_MISMATCH","dimension":"asset_health"})
    if dims["target_health_after"]["status"]!="MATCH": findings.append({"class":"AFTER_STATE_MISMATCH","dimension":"asset_health"})
 # per-task resource validation for every task actually present.
 try:
  summary=json.loads((P/"canonical"/f"{case}-summary.json").read_text())
  ext_rows=list(__import__("csv").DictReader((P/"external"/f"N12-{case}-fmpy.csv").open())); last=ext_rows[-1]
  tasks=int(spec["taskCount"])
  for index in range(tasks):
   mask=int(spec[f"t{index+1}RobotMask"])
   ctask=canonical_task_resources(case)[index]
   crobot=next((r for r in ctask if r.startswith("robot-")),None)
   if crobot is None: findings.append({"class":"RESOURCE_ELIGIBILITY_VIOLATION","model":"canonical","task":index})
   else:
    ci=int(crobot.split("robot-m")[1])
    if not (mask & (1 << (ci-1))): findings.append({"class":"RESOURCE_ELIGIBILITY_VIOLATION","model":"canonical","task":index})
   xr=int(ev[0]["after"][0]) if index==0 else int(float(last[f"t{index+1}Robot"]))
   if xr<=0 or not (mask & (1 << (xr-1))): findings.append({"class":"RESOURCE_ELIGIBILITY_VIOLATION","model":"external","task":index})
   required=int(spec[f"t{index+1}RequiresTool"])>0
   if required and "tool-t1" not in ctask: findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","model":"canonical","task":index})
   if required and not (int(float(last["toolTaskMask"])) & (1 << index)): findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","model":"external","task":index})
 except Exception as exc: findings.append({"class":"NOT_COMPARABLE","reason":f"resource selection not observed: {exc}"})
 # non-observable canonical dimensions
 dims["repair_state_before"]={"canonical":"NOT_OBSERVED","external":ev[0]["before"][4],"status":"NOT_COMPARABLE"}
 dims["restoration_committed"]={"canonical":"NOT_OBSERVED","external":ev[-1]["after"][5],"status":"NOT_COMPARABLE"}
 # same-time ordering among common events
 for i,a in enumerate(ce):
  for b in ce[i+1:]:
   if abs(a["at_s"]-b["at_s"])<=1e-9 and a["event_kind"] in xmap and b["event_kind"] in xmap:
    xa=xmap[a["event_kind"]][0]; xb=xmap[b["event_kind"]][0]
    if (a["event_sequence"]<b["event_sequence"]) != (xa["seq"]<xb["seq"]):
     findings.append({"class":"EVENT_ORDER_MISMATCH","at_s":a["at_s"]})
 # restoration precondition
 rs=[e for e in ev if e["kind"]=="RestorationCommitted"]
 for e in rs:
  if e["before"][4]!=1: findings.append({"class":"RESTORATION_PRECONDITION_VIOLATION"})
  if e["after"][5]!=1: findings.append({"class":"STATE_TRANSITION_MISMATCH","event":"RestorationCommitted"})
 if len(cmap.get("RepairCompleted",[]))==1 and len(xmap.get("RepairCompleted",[]))==1:
  c=cmap["RepairCompleted"][0]; x=xmap["RepairCompleted"][0]
  if abs(c["at_s"]-x["at"])>1e-9: findings.append({"class":"EVENT_TIME_DISPLACEMENT","event":"RepairCompleted"})
 # generic policy classification
 classification="SAME_CAUSAL_TRANSITION"
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ev)
 c_robot=canonical_robot(case)
 x_robot=int(ev[0]["after"][0])
 policy=(c_int and not x_int and c_robot is not None and c_robot!=x_robot and target==f"robot-m{c_robot}")
 if policy: classification="VALID_POLICY_DEPENDENT_DIVERGENCE"
 status="ACCEPTED" if not findings else "FAIL"
 return {"schema":"poie.s5v-n12-projection/1","case":case,"target":target,
         "canonical_events":ce,"external_events":ev,
         "dimensions":dims,"findings":findings,"classification":classification,"status":status,
         "first_identity_divergence":next((f for f in findings if f["class"]=="EVENT_TARGET_IDENTITY_MISMATCH"),None),
         "first_time_divergence":next((f for f in findings if f["class"]=="EVENT_TIME_DISPLACEMENT"),None),
         "first_order_divergence":next((f for f in findings if f["class"]=="EVENT_ORDER_MISMATCH"),None),
         "first_before_state_divergence":next((f for f in findings if f["class"]=="BEFORE_STATE_MISMATCH"),None),
         "first_after_state_divergence":next((f for f in findings if f["class"]=="AFTER_STATE_MISMATCH"),None),
         "first_unexplained_divergence":findings[0] if findings else None}
def main():
 results=[compare(c) for c in CASES]
 (P/"comparisons/projection.json").write_text(json.dumps({"schema":"poie.s5v-n12-projection-set/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["classification"],r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

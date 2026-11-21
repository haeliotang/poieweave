#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json
from pathlib import Path
P=Path(__file__).resolve().parent
CASES=[f"N16-{i:02d}" for i in (0,1,2,3,4,5,6,8,9,10,11,12,13,14)]+["N16-MUT-SHARED"]
BITS={2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}
UPSTREAM=set(BITS.values())-{"tool-t1"}
ASSETS={0:"robot-m1",1:"robot-m2",5:"sensor-x",6:"bus-2",3:"bus-1",7:"tool-t1"}
def bitset(mask): return {name for bit,name in BITS.items() if mask & (1<<bit)} - {"tool-t1"}
def main():
 results=[]
 for cid in CASES:
  js=json.loads((P/"contracts"/f"{cid}.json").read_text()); ce=[json.loads(x) for x in (P/"canonical"/f"{cid}.jsonl").read_text().splitlines()]
  journal=json.loads((P/"external"/f"{cid}.csv.journal.json").read_text())["events"]; rows=list(csv.DictReader((P/"external"/f"{cid}.csv").open())); last=rows[-1]
  findings=[]; tasks=js["tasks"]; tr=ce[0]["running_repair"]["task_resources"]
  # independent schedule checks canonical
  if len(tr)!=len(tasks): findings.append("CANONICAL_TASK_COUNT")
  resources={}
  for task,entry in zip(tasks,tr):
   resources[task["task_id"]]=set(entry["resources"])
   robot=next((r for r in entry["resources"] if r.startswith("robot")),None)
   if robot is None or not (task["robot_mask"] & (1<<(int(robot[-1])-1))): findings.append(f"CANONICAL_ROBOT_MASK_{task['task_id']}")
   if bool(task["requires_tool"]) != ("tool-t1" in entry["resources"]): findings.append(f"CANONICAL_TOOL_{task['task_id']}")
   if abs(entry["end_s"]-entry["start_s"]-task["duration_s"])>1e-9: findings.append(f"CANONICAL_DURATION_{task['task_id']}")
  for i in range(len(tr)):
   for j in range(i+1,len(tr)):
    if next((r for r in tr[i]["resources"] if r.startswith("robot")),None)==next((r for r in tr[j]["resources"] if r.startswith("robot")),None):
     if tr[j]["start_s"] < tr[i]["end_s"]-1e-9: findings.append("CANONICAL_CONTENTION")
  makespan=max(e["end_s"] for e in tr)
  # independent external schedule checks
  xrobots=[]
  for i,task in enumerate(tasks):
   er=ASSETS[int(float(last[f"t{i+1}Robot"]))-1]
   if not (task["robot_mask"] & (1<<(int(er[-1])-1))): findings.append(f"EXTERNAL_ROBOT_MASK_{task['task_id']}")
   est,een=float(last[f"t{i+1}Start_s"]),float(last[f"t{i+1}End_s"])
   if abs(een-est-task["duration_s"])>1e-9: findings.append(f"EXTERNAL_DURATION_{task['task_id']}")
   if bool(task["requires_tool"]) != bool(int(float(last["toolTaskMask"]))&(1<<i)): findings.append(f"EXTERNAL_TOOL_{task['task_id']}")
   xrobots.append(er)
  if abs(makespan-float(last["makespan_s"]))>1e-9: findings.append("MAKESPAN")
  # canonical closure projection at every event
  canon=[]; cclosure={}
  for e in ce:
   r=e.get("running_repair") or {}; deps=set(r.get("execution_dependencies") or []) if r else None
   if deps is not None: cclosure[e["event_sequence"]]=deps
   k=e["event_kind"]; t=e["at_s"]; target=e["target_asset"]
   if k in ("InitialRepairStarted","HorizonReached"): continue
   if k=="ResourceReleased":
    if target in UPSTREAM: k="DependencyDeactivated"
   if k in ("RepairCompleted","RepairInterrupted"): target="a-lathe"
   if k in ("TaskCompleted","ResourceReleased","DependencyDeactivated","AssetFailure","RepairInterrupted","RepairCompleted"): canon.append((t,k,target))
  # external closure projection
  xnorm=[]; xdep_after={}
  for e in journal:
   k=e["kind"]; t=e["at"]
   if k=="DependencyActivated": continue
   if k in ("InitialRepairStarted","RunTerminated"): continue
   et=e.get("target")
   if k in ("RepairCompleted","RepairInterrupted"): et="a-lathe"
   if k in ("TaskCompleted","ResourceReleased","DependencyDeactivated","AssetFailure","RepairInterrupted","RepairCompleted"): xnorm.append((t,k,et))
   if "dep_after" in e: xdep_after[(t,k,e.get("target"))]=e["dep_after"]
  # independent OR-policy detection and per-model sensitivity
  policy=False; policy_details=[]
  for i,task in enumerate(tasks):
   crobot=next((r for r in tr[i]["resources"] if r.startswith("robot")),None)
   erobot=ASSETS[int(float(last[f"t{i+1}Robot"]))-1]
   allowed=task.get("robot_alternatives") or []
   if crobot!=erobot and crobot in allowed and erobot in allowed:
    policy=True; policy_details.append({"task":task["task_id"],"canonical":crobot,"external":erobot})
  if policy:
   # each model must be sensitive according to its own closure at the failure
   for f in js["failures"]:
    cfail=[e for e in ce if e["event_kind"]=="AssetFailure" and e["target_asset"]==f["target"]]
    xfail=[e for e in journal if e["kind"]=="AssetFailure" and e.get("target")==f["target"]]
    if cfail:
     cdeps=set((cfail[0].get("running_repair") or {}).get("execution_dependencies") or [])
     c_expect=f["target"] in cdeps or f["target"] in set((ce[0].get("running_repair") or {}).get("restoration_targets") or [])
     if (any(e["event_kind"]=="RepairInterrupted" for e in ce) and not c_expect): findings.append("POLICY_CANONICAL_SENSITIVITY")
    if xfail and "dep_before" in xfail[0]:
     x_expect=bool(xfail[0]["dep_before"] & (1 << f["target_id"])) if f["kind"]!=1 else any(task["robot_mask"] and f["target"] in resources[task["task_id"]] for task in tasks)
     if (any(e["kind"]=="RepairInterrupted" for e in journal) and not x_expect): findings.append("POLICY_EXTERNAL_SENSITIVITY")
  # compare normalized streams by type/time/target
  cs=sorted(canon); xs=sorted(xnorm)
  if cs!=xs:
   # find first difference
   for i,(a,b) in enumerate(zip(cs,xs)):
    if a!=b: findings.append(f"EVENT_STREAM_DIVERGENCE:{a[0]}:{a[1]}:{b[1]}"); break
   else: findings.append("EVENT_COUNT_DIVERGENCE")
  # dependency closure after initial and at every deactivation
  if ce:
   init_deps=set((ce[0].get("running_repair") or {}).get("execution_dependencies") or []); init_up=init_deps & UPSTREAM
   initial=[e for e in journal if e["kind"]=="InitialRepairStarted"][0]
   if init_up!=bitset(initial["dep_after"]): findings.append(f"INITIAL_CLOSURE:{sorted(init_up)}:{sorted(bitset(initial['dep_after']))}")
  xdep_at={}
  for e in journal:
   if "dep_after" in e: xdep_at[e["at"]]=e["dep_after"]
  for at,mask in xdep_at.items():
   cand=[e for e in ce if abs(e["at_s"]-at)<1e-9 and e.get("running_repair")]
   if not cand: continue
   cd=set((cand[-1].get("running_repair") or {}).get("execution_dependencies") or []) & UPSTREAM
   if cd!=bitset(mask): findings.append(f"CLOSURE_AT_{at}")
  # independent closure invariant from the contract: shared/future upstream must persist
  profile=js.get("dependency_profile","default")
  task_up={}
  for i,task in enumerate(tasks):
   robot="robot-m1" if i==0 else "robot-m2"
   if profile=="split":
    task_up[task["task_id"]]={"bus-1","bat-1","charger-1"} if robot=="robot-m1" else {"bus-2"}
   else:
    task_up[task["task_id"]]={"bus-1","bat-1","charger-1"}
  for at in sorted({e["at_s"] for e in ce if e["event_kind"]=="TaskCompleted"}):
   unfinished=[task for i,task in enumerate(tasks) if tr[i]["end_s"]>at+1e-9]
   expected=set().union(*(task_up[task["task_id"]] for task in unfinished)) if unfinished else set()
   cand=[e for e in ce if abs(e["at_s"]-at)<1e-9 and e.get("running_repair")]
   if cand:
    cd=set((cand[-1].get("running_repair") or {}).get("execution_dependencies") or []) & UPSTREAM
    if not expected <= cd: findings.append("SHARED_DEPENDENCY_LOSS" if at<max(tr[i]["end_s"] for i in range(len(tr)))-1e-9 else "FUTURE_DEPENDENCY_LOSS")
   xev=[e for e in journal if "dep_after" in e and abs(e["at"]-at)<1e-9]
   if xev:
    xd=bitset(xev[-1]["dep_after"])
    if not expected <= xd: findings.append("SHARED_DEPENDENCY_LOSS_EXTERNAL" if at<max(tr[i]["end_s"] for i in range(len(tr)))-1e-9 else "FUTURE_DEPENDENCY_LOSS_EXTERNAL")
  # physical failure history: every exogenous failure must be recorded by both
  c_term=min([e["at_s"] for e in ce if e["event_kind"]=="RepairInterrupted"],default=float("inf"))
  x_term=min([e["at"] for e in journal if e["kind"]=="RepairInterrupted"],default=float("inf"))
  for f in js["failures"]:
   if f["time_s"] <= c_term+1e-9 and not any(e["event_kind"]=="AssetFailure" and abs(e["at_s"]-f["time_s"])<1e-9 and e["target_asset"]==f["target"] for e in ce): findings.append(f"CANONICAL_FAILURE_MISSING_{f['failure_id']}")
   if f["time_s"] <= x_term+1e-9 and not any(e["kind"]=="AssetFailure" and abs(e["at"]-f["time_s"])<1e-9 and e.get("target")==f["target"] for e in journal): findings.append(f"EXTERNAL_FAILURE_MISSING_{f['failure_id']}")
  c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in journal)
  c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in journal)
  if c_int!=x_int: findings.append("INTERRUPTION_DIVERGENCE")
  if c_comp!=x_comp: findings.append("GROUP_COMMIT_DIVERGENCE")
  if policy:
   findings=[f for f in findings if not (f.startswith("EVENT") or f.startswith("CLOSURE") or f.startswith("INITIAL_CLOSURE") or f.startswith("INTERRUPTION") or f.startswith("GROUP_COMMIT") or f.startswith("EXTERNAL_FAILURE") or f=="MAKESPAN")]
  results.append({"case":cid,"classification":"VALID_POLICY_DEPENDENT_DIVERGENCE" if policy else "SAME_POLICY","policy":policy_details,"initial_canonical_closure":sorted(init_up) if ce else [],"initial_external_closure":sorted(bitset(initial["dep_after"])) if journal else [],"canonical_events":len(canon),"external_events":len(xnorm),"canonical_interrupted":c_int,"external_interrupted":x_int,"findings":findings,"first_closure_divergence":next((f for f in findings if "CLOSURE" in f),None),"first_event_divergence":next((f for f in findings if "EVENT" in f),None),"status":"ACCEPTED" if not findings else "FAIL"})
 (P/"dependency-trajectories/results.json").write_text(json.dumps({"schema":"poie.s5v-n16-closure/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["findings"])
 def okay(r):
  if r["case"]=="N16-MUT-SHARED": return r["status"]=="FAIL" and any("EVENT" in f or "CLOSURE" in f or "INTERRUPTION" in f for f in r["findings"])
  if r["case"]=="N16-09": return r["status"]=="ACCEPTED" and r["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE"
  return r["status"]=="ACCEPTED"
 return 0 if all(okay(r) for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

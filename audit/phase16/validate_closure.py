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
   if k in ("TaskCompleted","ResourceReleased","DependencyDeactivated","AssetFailure","RepairInterrupted","RepairCompleted"): xnorm.append((t,k,et,e.get("dep_after")))
   if "dep_after" in e: xdep_after[(t,k,e.get("target"))]=e["dep_after"]
  # independent OR-policy detection and per-model sensitivity
  crobots=[next((r for r in tr[i]["resources"] if r.startswith("robot")),None) for i in range(len(tasks))]
  policy=False; policy_details=[]
  for i,task in enumerate(tasks):
   crobot=next((r for r in tr[i]["resources"] if r.startswith("robot")),None)
   erobot=ASSETS[int(float(last[f"t{i+1}Robot"]))-1]
   allowed=task.get("robot_alternatives") or []
   if crobot!=erobot and crobot in allowed and erobot in allowed:
    policy=True; policy_details.append({"task":task["task_id"],"canonical":crobot,"external":erobot})
  def upstream_for(robot,profile):
   if profile=="split":
    return {"bus-1","bat-1","charger-1"} if robot=="robot-m1" else {"bus-2"}
   return {"bus-1","bat-1","charger-1"}
  profile=js.get("dependency_profile","default")
  c_task_up={tasks[i]["task_id"]:upstream_for(next(r for r in tr[i]["resources"] if r.startswith("robot")),profile) for i in range(len(tasks))}
  x_task_up={tasks[i]["task_id"]:upstream_for(xrobots[i],profile) for i in range(len(tasks))}
  def closure_check(events, ends, robots, task_up_map, side):
   for at in sorted({e["at_s"] for e in events if e.get("event_kind")=="TaskCompleted"}):
    unfinished=[task for task in tasks if ends[task["task_id"]]>at+1e-9]
    expected=set().union(*(task_up_map[task["task_id"]] for task in unfinished)) if unfinished else set()
    cand=[e for e in events if abs(e["at_s"]-at)<1e-9 and e.get("running_repair")]
    if cand:
     actual=set((cand[-1].get("running_repair") or {}).get("execution_dependencies") or []) & UPSTREAM
     if not expected <= actual: findings.append(f"{side}_SHARED_DEPENDENCY_LOSS")
   # external native deactivation must not remove a still-needed upstream
   if side=="EXTERNAL":
    for de in [e for e in events if e.get("kind")=="DependencyDeactivated"]:
     if de.get("target") in UPSTREAM:
      unfinished=[task for task in tasks if ends[task["task_id"]]>de["at"]+1e-9]
      expected=set().union(*(task_up_map[task["task_id"]] for task in unfinished)) if unfinished else set()
      if de["target"] in expected: findings.append(f"EXTERNAL_PREMATURE_DEACTIVATION_{de['target']}")
  if policy:
   c_ends={task["task_id"]:tr[i]["end_s"] for i,task in enumerate(tasks)}
   x_ends={task["task_id"]:float(last[f"t{i+1}End_s"]) for i,task in enumerate(tasks)}
   closure_check([{"at_s":e["at_s"],"event_kind":e["event_kind"],"running_repair":e.get("running_repair")} for e in ce],c_ends,crobots,c_task_up,"CANONICAL")
   # external closure check uses dep_after vs expected at each task completion time
   for at in sorted({x_ends[t] for t in x_ends}):
    unfinished=[task for task in tasks if x_ends[task["task_id"]]>at+1e-9]
    expected=set().union(*(x_task_up[task["task_id"]] for task in unfinished)) if unfinished else set()
    cand=[e for e in journal if abs(e["at"]-at)<1e-9 and "dep_after" in e]
    if cand:
     actual=bitset(cand[-1]["dep_after"])
     if not expected <= actual: findings.append("EXTERNAL_SHARED_DEPENDENCY_LOSS")
  init_deps=set((ce[0].get("running_repair") or {}).get("execution_dependencies") or []) if ce else set(); init_up=init_deps & UPSTREAM
  initial=[e for e in journal if e["kind"]=="InitialRepairStarted"][0] if journal else {"dep_after":0}
  # compare normalized streams in ORIGINAL ORDER
  rank={"TaskCompleted":0,"ResourceReleased":1,"DependencyDeactivated":2,"AssetFailure":3,"RepairInterrupted":4,"RepairCompleted":3,"AssetFailure":4,"RepairInterrupted":5,"RestorationCommitted":6}
  def rank_ok(events):
   by={}
   for ev in events: by.setdefault(ev[0],[]).append(rank.get(ev[1],9))
   return all(v==sorted(v) for v in by.values())
  if not policy:
   if not rank_ok(canon): findings.append("CANONICAL_EVENT_ORDER")
   if not rank_ok(xnorm): findings.append("EXTERNAL_EVENT_ORDER")
   ct={}
   for e in canon: ct.setdefault(e[0],[]).append((e[1],e[2]))
   xt={}
   for e in xnorm: xt.setdefault(e[0],[]).append((e[1],e[2]))
   for at in sorted(set(ct)|set(xt)):
    if sorted(ct.get(at,[]))!=sorted(xt.get(at,[])): findings.append(f"EVENT_SET_DIVERGENCE_{at}")
  # independent closure invariant from the contract: shared/future upstream must persist
  task_up=c_task_up
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
  x_makespan=max(float(last[f"t{i+1}End_s"]) for i in range(len(tasks)))
  if any(e["event_kind"]=="RepairCompleted" and e["at_s"]<makespan-1e-9 for e in ce): findings.append("PREMATURE_GROUP_COMMIT")
  if any(e["kind"]=="RepairCompleted" and e["at"]<x_makespan-1e-9 for e in journal): findings.append("PREMATURE_GROUP_COMMIT_EXTERNAL")
  if x_int and any(e["kind"]=="RestorationCommitted" for e in journal): findings.append("INTERRUPTED_RESTORED")
  if any(e["kind"]=="RestorationCommitted" and e["at"]<x_makespan-1e-9 for e in journal): findings.append("PREMATURE_GROUP_COMMIT_EXTERNAL")
  if c_int!=x_int: findings.append("INTERRUPTION_DIVERGENCE")
  if c_comp!=x_comp: findings.append("GROUP_COMMIT_DIVERGENCE")
  if policy:
   findings=[f for f in findings if not (f.startswith("EVENT") or f.startswith("INTERRUPTION") or f.startswith("GROUP_COMMIT") or f=="MAKESPAN")]
  results.append({"case":cid,"classification":"VALID_POLICY_DEPENDENT_DIVERGENCE" if policy else "SAME_POLICY","policy":policy_details,"initial_canonical_closure":sorted(init_up) if ce else [],"initial_external_closure":sorted(bitset(initial["dep_after"])) if journal else [],"canonical_events":len(canon),"external_events":len(xnorm),"canonical_interrupted":c_int,"external_interrupted":x_int,"findings":findings,"first_closure_divergence":next((f for f in findings if "CLOSURE" in f),None),"first_event_divergence":next((f for f in findings if "EVENT" in f),None),"status":"ACCEPTED" if not findings else "FAIL"})
 (P/"dependency-trajectories/results.json").write_text(json.dumps({"schema":"poie.s5v-n16-closure/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["findings"])
 def okay(r):
  if r["case"]=="N16-MUT-SHARED": return r["status"]=="FAIL" and any("EVENT" in f or "CLOSURE" in f or "INTERRUPTION" in f for f in r["findings"])
  if r["case"]=="N16-09": return r["status"]=="ACCEPTED" and r["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE"
  return r["status"]=="ACCEPTED"
 return 0 if all(okay(r) for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

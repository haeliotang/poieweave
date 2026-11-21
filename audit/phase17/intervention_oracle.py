#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent per-model and counterfactual oracle for N17 interventions."""
from __future__ import annotations
import csv,json
from pathlib import Path
UP={"bus-1","bus-2","bat-1","charger-1"}
BITS={2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2"}
ASSETS={0:"robot-m1",1:"robot-m2",5:"sensor-x",6:"bus-2",3:"bus-1",7:"tool-t1"}
def upstream_for(robot,profile):
 if profile=="split":
  return {"bus-1","bat-1","charger-1"} if robot=="robot-m1" else {"bus-2"}
 return {"bus-1","bat-1","charger-1"}
def bits(mask): return {n for b,n in BITS.items() if mask&(1<<b)}
def model_outcome(spec,side,json_path,contract_path,run_dir):
 js=json.loads(Path(json_path).read_text()); root=Path(run_dir); cid=Path(json_path).stem
 findings=[]; tasks=js["tasks"]; profile=js.get("dependency_profile","default")
 if side=="canonical":
  ce=[json.loads(x) for x in (root/"canonical"/f"{cid}.jsonl").read_text().splitlines()]
  tr=ce[0]["running_repair"]["task_resources"]
  if len(tr)!=len(tasks): findings.append("TASK_COUNT")
  robots=[]; ends={}; resources={}
  for i,task in enumerate(tasks):
   e=tr[i]; selected=e["resources"]; resources[task["task_id"]]=set(selected)
   robot=next((r for r in selected if r.startswith("robot")),None); robots.append(robot)
   if robot is None or not (task["robot_mask"] & (1<<(int(robot[-1])-1))): findings.append(f"ROBOT_MASK_{task['task_id']}")
   if bool(task["requires_tool"]) != ("tool-t1" in selected): findings.append(f"TOOL_{task['task_id']}")
   if abs(e["end_s"]-e["start_s"]-task["duration_s"])>1e-9: findings.append(f"DURATION_{task['task_id']}")
   ends[task["task_id"]]=e["end_s"]
  for i in range(len(tr)):
   for j in range(i+1,len(tr)):
    if robots[i]==robots[j] and tr[j]["start_s"]<tr[i]["end_s"]-1e-9: findings.append("CONTENTION")
  makespan=max(ends.values())
  failures=[(e["at_s"],e["target_asset"]) for e in ce if e["event_kind"]=="AssetFailure"]
  interruption=next((e["at_s"] for e in ce if e["event_kind"]=="RepairInterrupted"),None)
  completed=any(e["event_kind"]=="RepairCompleted" for e in ce)
  restoration=None
  closure_at={}
  for e in ce:
   if e.get("running_repair") and e["at_s"]==ends.get("A"): closure_at["A"]=set(e["running_repair"]["execution_dependencies"])&UP
  for at in {ends[t] for t in ends}:
   expected={task["task_id"]:upstream_for(robots[i],profile) for i,task in enumerate(tasks)}
   unfinished=[task for task in tasks if ends[task["task_id"]]>at+1e-9]
   exp=set().union(*(expected[task["task_id"]] for task in unfinished)) if unfinished else set()
   evs=[e for e in ce if abs(e["at_s"]-at)<1e-9 and e.get("running_repair")]
   if evs:
    actual=set(evs[-1]["running_repair"]["execution_dependencies"])&UP
    if not exp<=actual: findings.append(f"CLOSURE_{at}")
  termination=interruption if interruption is not None else (makespan if completed else None)
 else:
  ev=json.loads((root/"external"/f"{cid}.csv.journal.json").read_text())["events"]; last=list(csv.DictReader((root/"external"/f"{cid}.csv").open()))[-1]
  robots=[ASSETS[int(float(last[f"t{i+1}Robot"]))-1] for i in range(len(tasks))]
  ends={}; resources={}
  for i,task in enumerate(tasks):
   ends[task["task_id"]]=float(last[f"t{i+1}End_s"]); resources[task["task_id"]]=set()
   if not (task["robot_mask"] & (1<<(int(robots[i][-1])-1))): findings.append(f"ROBOT_MASK_{task['task_id']}")
   if bool(task["requires_tool"]) != bool(int(float(last["toolTaskMask"]))&(1<<i)): findings.append(f"TOOL_{task['task_id']}")
   if abs(float(last[f"t{i+1}End_s"])-float(last[f"t{i+1}Start_s"])-task["duration_s"])>1e-9: findings.append(f"DURATION_{task['task_id']}")
  for i in range(len(tasks)):
   for j in range(i+1,len(tasks)):
    if robots[i]==robots[j] and float(last[f"t{j+1}Start_s"])<float(last[f"t{i+1}End_s"])-1e-9: findings.append("CONTENTION")
  makespan=max(ends.values()); failures=[(e["at"],e.get("target")) for e in ev if e["kind"]=="AssetFailure"]
  interruption=next((e["at"] for e in ev if e["kind"]=="RepairInterrupted"),None)
  completed=any(e["kind"]=="RepairCompleted" for e in ev)
  restoration=next((e["at"] for e in ev if e["kind"]=="RestorationCommitted"),None)
  for at in {ends[t] for t in ends}:
   unfinished=[task for task in tasks if ends[task["task_id"]]>at+1e-9]
   exp=set().union(*(upstream_for(robots[i],profile) for i,task in enumerate(tasks) if task["task_id"] in {x["task_id"] for x in unfinished})) if unfinished else set()
   evs=[e for e in ev if abs(e["at"]-at)<1e-9 and "dep_after" in e]
   if evs:
    actual=bits(evs[-1]["dep_after"])
    if not exp<=actual: findings.append(f"CLOSURE_{at}")
  termination=interruption if interruption is not None else (makespan if completed else None)
 # expected sensitivity from actual selected robots
 first=None
 for time,target in sorted(failures):
  if termination is not None and time>termination+1e-9: continue
  hit=False
  for i,task in enumerate(tasks):
   if time<ends[task["task_id"]]-1e-9:
    if target==robots[i] or (target=="tool-t1" and task["requires_tool"]) or target in upstream_for(robots[i],profile): hit=True
  if hit: first=(time,target); break
 if first is not None:
  if interruption is None: findings.append("ACTIVE_DEPENDENCY_FAILURE_IGNORED")
  elif abs(interruption-first[0])>1e-9: findings.append("INTERRUPTION_TIME")
  if restoration is not None: findings.append("RESTORED_AFTER_INTERRUPTION")
 else:
  if interruption is not None: findings.append("FALSE_INTERRUPTION")
  if not completed: findings.append("MISSING_GROUP_COMMIT")
  if completed and restoration is not None and restoration<makespan-1e-9: findings.append("PREMATURE_RESTORATION")
 # scheduled failures must be recorded (unless after termination)
 for f in js["failures"]:
  if termination is not None and f["time_s"]>termination+1e-9: continue
  if not any(abs(t-f["time_s"])<1e-9 and a==f["target"] for t,a in failures): findings.append(f"FAILURE_MISSING_{f['failure_id']}")
 return {"side":side,"valid":not findings,"findings":findings,"selected_robots":robots,"selected_resources":{t:sorted(r) for t,r in resources.items()},"makespan":makespan,"failures":failures,"interruption":interruption,"completed":completed,"restoration":restoration,"termination":termination,"expected_first_hit":first}
def relation(base_c,base_e,int_c,int_e,policy):
 valid=base_c["valid"] and base_e["valid"] and int_c["valid"] and int_e["valid"]
 if not valid: return "INVALID_EXECUTION",{}
 def sig(o): return (o["completed"],o["interruption"],tuple(sorted(o["failures"])),o["restoration"],round(o["makespan"],6),tuple(o["selected_robots"]))
 dc=sig(base_c)!=sig(int_c); de=sig(base_e)!=sig(int_e)
 rel="SAME_OUTCOME_VALID"
 if dc or de:
  if policy and dc!=de: rel="VALID_POLICY_DEPENDENT_DIVERGENCE"
  elif dc and de: rel="CHANGED_OUTCOME_EXPLAINED"
  elif policy: rel="VALID_POLICY_DEPENDENT_DIVERGENCE"
  else: rel="UNEXPLAINED_CAUSAL_DIVERGENCE"
 return rel,{"canonical_changed":dc,"external_changed":de}

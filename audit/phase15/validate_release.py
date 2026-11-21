#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json
from pathlib import Path
P=Path(__file__).resolve().parent
CASES=["N15-D1-R1-7201","N15-D1-R2-7201","N15-D1-R1-7199","N15-D2-SERIAL-7201","N15-D3-TOOL-7201","N15-D3-TOOL-7199","N15-OR-RELEASE","N15-MUT-ORIG"]
ASSETS={0:"robot-m1",1:"robot-m2",7:"tool-t1"}
def main():
 results=[]
 for cid in CASES:
  js=json.loads((P/"contracts"/f"{cid}.json").read_text()); ce=[json.loads(x) for x in (P/"canonical"/f"{cid}.jsonl").read_text().splitlines()]
  journal=json.loads((P/"external"/f"{cid}.csv.journal.json").read_text())["events"]
  last=list(csv.DictReader((P/"external"/f"{cid}.csv").open()))[-1]
  findings=[]; tasks=js["tasks"]
  # task lifecycle / releases expected from canonical selected resources
  tr=ce[0]["running_repair"]["task_resources"]; resources={t["task_id"]:set(e["resources"]) for t,e in zip(tasks,tr)}
  ends={t["task_id"]:e["end_s"] for t,e in zip(tasks,tr)}
  makespan=max(ends.values())
  xf=[e for e in journal if e["kind"]=="AssetFailure"]
  def hit(target,time):
   for task in tasks:
    if time < ends[task["task_id"]]-1e-9 and target in resources[task["task_id"]]: return True
   return False
  first_hit=min((e["at"] for e in xf if hit(e["target"],e["at"])),default=None)
  expected_interrupt=first_hit is not None
  expected_completed={tid for tid,end in ends.items() if first_hit is None or end < first_hit-1e-9}
  completed={e["target_asset"]:e["at_s"] for e in ce if e["event_kind"]=="TaskCompleted"}
  releases={e["target_asset"]:e["at_s"] for e in ce if e["event_kind"]=="ResourceReleased"}
  if set(completed)!=expected_completed: findings.append("TASK_COMPLETION_MISSING")
  for tid in expected_completed:
   if abs(completed.get(tid,-1)-ends[tid])>1e-9: findings.append(f"TASK_COMPLETION_TIME_{tid}")
  for task in tasks:
   allowed=task.get("robot_alternatives",[])
   if allowed:
    selected=next((r for r in resources[task["task_id"]] if r.startswith("robot")),None)
    if selected not in allowed: findings.append(f"RESOURCE_ELIGIBILITY_VIOLATION_{task['task_id']}")
  for tid in expected_completed:
   end=ends[tid]
   for rid in resources.get(tid,set()):
    if rid not in ("robot-m1","robot-m2","tool-t1"): continue
    unfinished_users=[other for other in tasks if other["task_id"] not in expected_completed and rid in resources.get(other["task_id"],set())]
    if not unfinished_users and releases.get(rid) is None: findings.append(f"RESOURCE_RELEASE_MISSING_{rid}")
    if not unfinished_users and releases.get(rid) is not None and abs(releases[rid]-end)>1e-9: findings.append(f"RESOURCE_RELEASE_TIME_{rid}")
    if unfinished_users and releases.get(rid) is not None and releases[rid] < end+1e-9: findings.append(f"SHARED_RESOURCE_RELEASE_VIOLATION_{rid}")
  xt=[e for e in journal if e["kind"]=="TaskCompleted"]; xr=[e for e in journal if e["kind"]=="ResourceReleased"]
  if set(e["target"] for e in xt)!=expected_completed: findings.append("EXTERNAL_TASK_COMPLETION")
  # extra external release not present in canonical evidence
  for e in xr:
   if e["target"] in ("robot-m1","robot-m2","tool-t1") and e["target"] not in releases:
    findings.append(f"EXTERNAL_RELEASE_MISMATCH_{e['target']}")
  # compare release events for resources of completed tasks
  for rid,at in releases.items():
   if rid not in ("robot-m1","robot-m2","tool-t1"): continue
   xr_at=next((e["at"] for e in xr if e["target"]==rid),None)
   if xr_at is None or abs(xr_at-at)>1e-9: findings.append(f"EXTERNAL_RELEASE_MISMATCH_{rid}")
  # failure sensitivity
  exp_int=expected_interrupt
  c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in journal)
  if c_int!=exp_int: findings.append("CANONICAL_FAILURE_SENSITIVITY")
  if x_int!=exp_int: findings.append("EXTERNAL_FAILURE_SENSITIVITY")
  c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in journal)
  if c_comp!=x_comp: findings.append("GROUP_COMMIT_MISMATCH")
  if any(e["kind"]=="RestorationCommitted" for e in journal) and x_int: findings.append("INTERRUPTED_RESTORED")
  if any(e["event_kind"]=="RepairCompleted" and e["at_s"]<makespan-1e-9 for e in ce): findings.append("PREMATURE_GROUP_COMMIT")
  if x_comp and any(e["kind"]=="RestorationCommitted" and e["at"]<makespan-1e-9 for e in journal): findings.append("PREMATURE_GROUP_COMMIT")
  results.append({"case":cid,"canonical_completed":completed,"canonical_released":releases,"external_completed":[(e["target"],e["at"]) for e in xt],"external_released":[(e["target"],e["at"]) for e in xr],"canonical_interrupted":c_int,"external_interrupted":x_int,"makespan":makespan,"findings":findings,"first_release_divergence":next((f for f in findings if "RELEASE" in f),None),"first_sensitivity_divergence":next((f for f in findings if "SENSITIVITY" in f),None),"status":"ACCEPTED" if not findings else "FAIL"})
 (P/"resource-lifetimes/results.json").write_text(json.dumps({"schema":"poie.s5v-n15-release/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["findings"])
 def okay(r):
  if r["case"]=="N15-MUT-ORIG": return r["status"]=="FAIL" and any("EXTERNAL_RELEASE_MISMATCH" in f for f in r["findings"])
  return r["status"]=="ACCEPTED"
 return 0 if all(okay(r) for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

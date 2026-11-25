#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Bounded independent specification-level reference oracle (no model output)."""
from __future__ import annotations
import itertools,json
DUR={"A":7200.0,"B":10800.0}
UP={"bus-1","bus-2","bat-1","charger-1"}
def upstream(robot,profile):
 if profile=="split": return {"bus-1","bat-1","charger-1"} if robot=="robot-m1" else {"bus-2"}
 return {"bus-1","bat-1","charger-1"}
def robots_for(task):
 return ["robot-m2" if b=="2" else "robot-m1" for b in task["mask"]]
def schedule_candidates(rA,rB,tool):
 raw=[]
 if rA!=rB: raw.append((0.0,0.0))
 raw += [(0.0,7200.0),(10800.0,0.0)]
 out=[]
 for sA,sB in raw:
  eA,eB=sA+7200.0,sB+10800.0
  overlap=not (eA<=sB+1e-9 or eB<=sA+1e-9)
  shared=(rA==rB) or tool
  if overlap and shared: continue
  if max(eA,eB)>72000.0: continue
  out.append((sA,sB))
 return out
def predict(spec):
 profile=spec["profile"]; tool=spec["tool"]; failures=spec["failures"]
 tasks={"A":{"id":"A","robots":spec["a_robots"],"dur":7200.0,"tool":tool},"B":{"id":"B","robots":spec["b_robots"],"dur":10800.0,"tool":tool}}
 out=[]
 for rA in tasks["A"]["robots"]:
  for rB in tasks["B"]["robots"]:
   for sA,sB in schedule_candidates(rA,rB,tool):
    ends={"A":sA+7200.0,"B":sB+10800.0}; starts={"A":sA,"B":sB}; makespan=max(ends.values())
    selected={"A":{rA}|({"tool-t1"} if tool else set()),"B":{rB}|({"tool-t1"} if tool else set())}
    active_deps={"A":{rA}|({"tool-t1"} if tool else set())|upstream(rA,profile),"B":{rB}|({"tool-t1"} if tool else set())|upstream(rB,profile)}
    interrupt=None; applied=[]
    for f in sorted(failures,key=lambda x:(x["time_s"],x["target"])):
     t=f["time_s"]
     if t>makespan+1e-9: continue
     # completion at the same timestamp precedes failure
     unfinished=[tid for tid in ("A","B") if ends[tid]>t+1e-9]
     hit=any(f["target"] in active_deps[tid] for tid in unfinished)
     if hit:
      interrupt=(t,f["target"]); applied.append(f); break
     applied.append(f)
    result={"robots":{"A":rA,"B":rB},"start_s":starts,"end_s":ends,"makespan_s":makespan,"selected_resources":{k:sorted(v) for k,v in selected.items()},"dependency_closure":{k:sorted(v) for k,v in active_deps.items()},"applied_failures":applied,"interruption":interrupt,"completed":interrupt is None,"termination":interrupt[0] if interrupt else makespan}
    out.append(result)
 # deduplicate by schedule/outcome
 uniq=[]; seen=set()
 for r in out:
  key=(r["robots"]["A"],r["robots"]["B"],r["start_s"]["A"],r["start_s"]["B"],r["completed"],r["interruption"])
  if key not in seen: seen.add(key); uniq.append(r)
 return {"schema":"poie.s5v-n21-reference-prediction/2","spec_hash":__import__("hashlib").sha256(json.dumps(spec,sort_keys=True).encode()).hexdigest(),"status":"POLICY_CONDITIONAL" if len({(r["robots"]["A"],r["robots"]["B"]) for r in uniq})>1 else "DETERMINISTIC","admissible_schedules":uniq}
def match_actual(pred, actual):
 """actual: task_id -> {robot,start,end} validated externally."""
 for r in pred["admissible_schedules"]:
  ok=True
  for tid in ("A","B"):
   a=actual[tid]
   if a["robot"]!=r["robots"][tid] or abs(a["start"]-r["start_s"][tid])>1e-9 or abs(a["end"]-r["end_s"][tid])>1e-9: ok=False; break
  if ok: return {"status":"MATCH","schedule":r}
 return {"status":"NO_ADMISSIBLE_MATCH","actual":actual}

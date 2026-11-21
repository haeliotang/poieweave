#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17"))
from intervention_oracle import model_outcome
N17=ROOT/"audit/phase17"
def outcome(model_dir,cid,label,spec,side):
 root=model_dir/cid; return model_outcome(spec,side,str(root/f"{cid}-{label}.json"),str(root/f"{cid}-{label}.contract"),str(root))
def sig(o): return {"completed":o["completed"],"interruption":o["interruption"],"restoration":o["restoration"],"makespan":round(o["makespan"],6),"robots":tuple(o["selected_robots"]),"failures":tuple(sorted(o["failures"]))}
EXPECTED={
 "N17-C1":{"outcome":True,"failure":True,"schedule":False},
 "N17-C2":{"outcome":True,"failure":True,"schedule":False},
 "N17-C3":{"outcome":True,"failure":True,"schedule":False},
 "N17-C4":{"outcome":False,"failure":False,"schedule":True},
 "N17-C6":{"outcome":False,"failure":True,"schedule":True},
 "N17-C7":{"outcome":False,"failure":True,"schedule":False},
 "N17-C8a":{"outcome":True,"failure":True,"schedule":False},
 "N17-C8b":{"outcome":False,"failure":True,"schedule":False},
 "N17-C9":{"outcome":False,"failure":True,"schedule":False},
 "N17-C10":{"outcome":True,"failure":True,"schedule":False},
}
def main():
 pairs=json.loads((N17/"oracle-results/fixed-pairs.json").read_text())["pairs"]; results=[]
 for p in pairs:
  cid=p["intervention_id"]; base=p["baseline"]; inter=p["intervention"]
  # specs reconstructed from stored contracts
  bspec=json.loads((N17/"baseline-runs"/cid/f"{cid}-baseline.json").read_text())
  ispec=json.loads((N17/"intervention-runs"/cid/f"{cid}-intervention.json").read_text())
  def sp(js):
   return {"profile":"split" if js["dependency_profile"]=="split" else "shared","tool":bool(js["tasks"][0]["requires_tool"]),"failures":[{"time_s":f["time_s"],"target":f["target"],"kind":f["kind"],"target_id":f["target_id"]} for f in js["failures"]],"alts":[t["robot_alternatives"] or None for t in js["tasks"]],"pref":js["preferred_robot"]}
  bs,is_=sp(bspec),sp(ispec)
  bc=outcome(N17/"baseline-runs",cid,"baseline",bs,"canonical"); be=outcome(N17/"baseline-runs",cid,"baseline",bs,"external")
  ic=outcome(N17/"intervention-runs",cid,"intervention",is_,"canonical"); ie=outcome(N17/"intervention-runs",cid,"intervention",is_,"external")
  exp=EXPECTED[cid]; findings=[]
  if not (bc["valid"] and be["valid"] and ic["valid"] and ie["valid"]): findings.append("INVALID_EXECUTION")
  oc=bc["completed"]!=ic["completed"] or (bc["interruption"] is None)!=(ic["interruption"] is None)
  oe=be["completed"]!=ie["completed"] or (be["interruption"] is None)!=(ie["interruption"] is None)
  outcome_changed=oc or oe
  failure_changed=tuple(sorted(bc["failures"]))!=tuple(sorted(ic["failures"])) or tuple(sorted(be["failures"]))!=tuple(sorted(ie["failures"]))
  schedule_changed=tuple(bc["selected_robots"])!=tuple(ic["selected_robots"]) or tuple(be["selected_robots"])!=tuple(ie["selected_robots"]) or abs(bc["makespan"]-ic["makespan"])>1e-9 or abs(be["makespan"]-ie["makespan"])>1e-9
  policy=any(tuple(bc["selected_robots"])[i]!=tuple(be["selected_robots"])[i] for i in range(2)) or any(tuple(ic["selected_robots"])[i]!=tuple(ie["selected_robots"])[i] for i in range(2))
  if exp["outcome"]:
   rel="CHANGED_REPAIR_OUTCOME_EXPLAINED" if outcome_changed else "EXPECTED_EFFECT_MISSING"
   if not outcome_changed: findings.append("EXPECTED_EFFECT_MISSING")
  else:
   if occurrence:=outcome_changed:
    rel="UNEXPECTED_REPAIR_OUTCOME_CHANGE"; findings.append(rel)
   elif failure_changed: rel="DIFFERENT_FAILURE_HISTORY_SAME_REPAIR_OUTCOME"
   elif schedule_changed: rel="CHANGED_SCHEDULE_EXPLAINED"
   else: rel="SAME_REPAIR_OUTCOME_VALID"
  if policy: rel="VALID_POLICY_DEPENDENT_DIVERGENCE" if not findings else rel
  results.append({"intervention_id":cid,"expected":exp,"dimensions":{"outcome_changed":outcome_changed,"failure_history_changed":failure_changed,"schedule_changed":schedule_changed,"policy_difference":policy},"previous_relation":p["relation"],"corrected_relation":rel,"findings":findings,"status":"ACCEPTED" if not findings and rel in ("CHANGED_REPAIR_OUTCOME_EXPLAINED","DIFFERENT_FAILURE_HISTORY_SAME_REPAIR_OUTCOME","CHANGED_SCHEDULE_EXPLAINED","SAME_REPAIR_OUTCOME_VALID","VALID_POLICY_DEPENDENT_DIVERGENCE") else "FAIL"})
 (P/"causal-comparisons/n17-revalidation.json").write_text(json.dumps({"schema":"poie.s5v-n18-n17-revalidation/1","pairs":results},indent=2)+"\n")
 for r in results: print(r["intervention_id"],r["previous_relation"],"->",r["corrected_relation"],r["status"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

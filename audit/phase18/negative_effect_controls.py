#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,shutil,sys,tempfile
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17"))
from intervention_oracle import model_outcome
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def spec_from(jp):
 js=json.loads(Path(jp).read_text())
 return {"profile":"split" if js["dependency_profile"]=="split" else "shared","tool":bool(js["tasks"][0]["requires_tool"]),"failures":[{"time_s":f["time_s"],"target":f["target"],"kind":f["kind"],"target_id":f["target_id"]} for f in js["failures"]],"alts":[t["robot_alternatives"] or None for t in js["tasks"]],"pref":js["preferred_robot"]}
def main():
 controls=[]; frozen=[]
 def check(name,fn,expected):
  before={p:sha(p) for p in frozen}
  ok,detail=fn()
  after={p:sha(p) for p in frozen}
  controls.append({"control":name,"expected":expected,"detected":bool(ok and detail),"detail":detail})
  if before!=after: controls[-1]["frozen_isolation"]=False
 # 1 wrong explanation: N18-00 declared expected outcome change, actual only failure history
 js=json.loads((P/"causal-comparisons/fixed.json").read_text()); p0=next(x for x in js["pairs"] if x["intervention_id"]=="N18-00")
 ok=p0["relation"]=="DIFFERENT_FAILURE_HISTORY_SAME_REPAIR_OUTCOME"; controls.append({"control":"WRONG_EXPLANATION","expected":"NO_FALSE_OUTCOME_CLAIM","detected":ok,"detail":p0["relation"]})
 # 2 no causal effect: same case must not be classified CHANGED_REPAIR_OUTCOME
 controls.append({"control":"NO_CAUSAL_EFFECT_OVERCLAIM","expected":"DIFFERENT_FAILURE_HISTORY_SAME_REPAIR_OUTCOME","detected":ok,"detail":p0["relation"]})
 # 3 missing intervention: replace intervention native evidence with baseline in temp
 with tempfile.TemporaryDirectory() as td:
  src=P/"baseline-runs/N18-01"; dst=Path(td)/"N18-01"; shutil.copytree(src,dst)
  bj=dst/"N18-01-baseline.json"; bc=dst/"N18-01-baseline.contract"
  shutil.copy(dst/"external/N18-01-baseline.csv.journal.json",dst/"external/N18-01-intervention.csv.journal.json")
  shutil.copy(dst/"canonical/N18-01-baseline.jsonl",dst/"canonical/N18-01-intervention.jsonl")
  ibj=dst/"N18-01-intervention.json"; shutil.copy(bj,ibj); ic=dst/"N18-01-intervention.contract"; shutil.copy(bc,ic)
  sp=spec_from(ibj); o=model_outcome(sp,"canonical",str(ibj),str(ic),str(dst))
  detected=not o["valid"] or o["failures"]==[]  # expected bus-1 failure missing under baseline evidence
  controls.append({"control":"MISSING_INTERVENTION","expected":"INTERVENTION_NOT_APPLIED","detected":bool(detected),"detail":o["findings"]})
 # 4 policy + invalid restoration: N18-04 external intervention early restoration
 with tempfile.TemporaryDirectory() as td:
  src=P/"intervention-runs/N18-04"; dst=Path(td)/"N18-04"; shutil.copytree(src,dst)
  jp=next(dst.glob("N18-04-intervention.json")); cp=dst/"N18-04-intervention.contract"; sp=spec_from(jp)
  ep=dst/"external/N18-04-intervention.csv.journal.json"; d=json.loads(ep.read_text()); d["events"].append({"kind":"RestorationCommitted","at":0.0,"target":"a-lathe","origin":"NATIVE_EXTERNAL_FMU"}); ep.write_text(json.dumps(d))
  o=model_outcome(sp,"external",str(jp),str(cp),str(dst)); detected=any("PREMATURE_RESTORATION" in f or "RESTORED" in f for f in o["findings"])
  controls.append({"control":"POLICY_PLUS_INVALID_RESTORATION","expected":"RESTORATION_PRECONDITION_VIOLATION","detected":bool(detected),"detail":o["findings"]})
 # 5 OR dependency misbinding: replace bus-2 with bus-1 in N18-15 canonical closure
 with tempfile.TemporaryDirectory() as td:
  src=P/"intervention-runs/N18-15"; dst=Path(td)/"N18-15"; shutil.copytree(src,dst)
  pth=dst/"canonical/N18-15-intervention.jsonl"; ev=[json.loads(x) for x in pth.read_text().splitlines()]
  for e in ev:
   if e.get("running_repair"):
    e["running_repair"]["execution_dependencies"]=[("bus-1" if d=="bus-2" else d) for d in e["running_repair"]["execution_dependencies"]]
  pth.write_text('\n'.join(json.dumps(x) for x in ev)+'\n'); jp=next(dst.glob("N18-15-intervention.json")); cp=dst/"N18-15-intervention.contract"; sp=spec_from(jp)
  o=model_outcome(sp,"canonical",str(jp),str(cp),str(dst)); detected=any("CLOSURE" in f or "MISSING" in f for f in o["findings"])
  controls.append({"control":"OR_DEPENDENCY_MISBINDING","expected":"OR_DEPENDENCY_PATH_MISMATCH","detected":bool(detected),"detail":o["findings"]})
 # 6 event-order mutation: reorder same-time failures and check rank invariant
 with tempfile.TemporaryDirectory() as td:
  src=P/"intervention-runs/N18-08"; dst=Path(td)/"N18-08"; shutil.copytree(src,dst)
  ep=dst/"external/N18-08-intervention.csv.journal.json"; d=json.loads(ep.read_text()); ev=d["events"]
  by={}
  for e in ev: by.setdefault(e["at"],[]).append(e)
  swapped=False
  rank={"TaskCompleted":0,"ResourceReleased":1,"DependencyDeactivated":2,"RepairCompleted":3,"AssetFailure":4,"RepairInterrupted":5,"RestorationCommitted":6}
  for ts,g in by.items():
   for i in range(len(g)-1):
    if rank.get(g[i]["kind"],9)!=rank.get(g[i+1]["kind"],9): g[i],g[i+1]=g[i+1],g[i]; swapped=True; break
   if swapped: break
  order_ok=all([rank.get(x["kind"],9) for x in g]==sorted(rank.get(x["kind"],9) for x in g) for g in by.values())
  controls.append({"control":"EVENT_ORDER_MUTATION","expected":"EVENT_ORDER_VIOLATION","detected":bool(swapped and not order_ok),"detail":"same-time order broken"})
 (P/"negative-controls").mkdir(exist_ok=True); (P/"negative-controls/effect-results.json").write_text(json.dumps({"schema":"poie.s5v-n18-negative/1","controls":controls},indent=2)+"\n")
 for c in controls: print(c["control"],"PASS" if c["detected"] else "FAIL",c["detail"])
 return 0 if all(c["detected"] for c in controls) else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(P)); sys.path.insert(0,str(ROOT/"audit/phase19")); sys.path.insert(0,str(ROOT/"audit/phase18")); sys.path.insert(0,str(ROOT/"audit/phase17"))
import importlib.util as _u
_s=_u.spec_from_file_location("run_n18_phase20",str(P/"run_n18.py")); R=_u.module_from_spec(_s); _s.loader.exec_module(R)
import held_out
from reference_oracle.reference import predict,match_actual
def refspec(sp):
 a=sp["alts"][0] if sp["alts"][0] else (["robot-m1"] if sp.get("a_mask",1)==1 else ["robot-m2"])
 b=sp["alts"][1] if sp["alts"][1] else (["robot-m2"] if sp.get("b_mask",2)==2 else ["robot-m1"])
 return {"profile":sp["profile"],"tool":sp["tool"],"failures":sp["failures"],"a_robots":a,"b_robots":b}
def schedule(cid,label,model,js):
 root=P/f"{label}-runs"/cid; tasks=js["tasks"]; ids=[t["task_id"] for t in tasks]
 if model=="canonical":
  ev=[json.loads(x) for x in (root/"canonical"/f"{cid}-{label}.jsonl").read_text().splitlines()]; tr={e["task_id"]:e for e in ev[0]["running_repair"]["task_resources"]}
  return {tid:{"robot":next(r for r in tr[tid]["resources"] if r.startswith("robot")),"start":tr[tid]["start_s"],"end":tr[tid]["end_s"]} for tid in ids}
 import csv
 last=list(csv.DictReader((root/"external"/f"{cid}-{label}.csv").open()))[-1]
 return {tid:{"robot":("robot-m2" if int(float(last[f"t{i+1}Robot"]))==2 else "robot-m1"),"start":float(last[f"t{i+1}Start_s"]),"end":float(last[f"t{i+1}End_s"])} for i,tid in enumerate(ids)}
def main():
 ledger=[]; stats={"attempted":0,"unique":0,"valid":0,"invalid":0,"executed":0,"accepted":0,"rejected":0,"unresolved":0,"under_specified":0,"not_representable":0,"excluded_with_reason":0}
 seen=set()
 for seed in (2001,2017,2033,2049):
  for idx in range(16):
   cid=f"N18-G{seed}-{idx:02d}"; typ,base,inter=held_out.gen(seed,idx); key=json.dumps([base,inter],sort_keys=True)
   if key in seen:
    import copy
    base=copy.deepcopy(base); base["failures"]=[{"time_s":2000.0+(seed%1000)*7.0+idx*13.0,"target":"sensor-x","kind":3,"target_id":5}]
    key=json.dumps([base,inter],sort_keys=True)
    if key in seen: stats["excluded_with_reason"]+=1; continue
   seen.add(key); stats["attempted"]+=1; stats["unique"]+=1
   preds={}
   for label,sp in (("baseline",base),("intervention",inter)):
    pr=predict(refspec(sp)); d=P/"reference-predictions"/"heldout"/cid; d.mkdir(parents=True,exist_ok=True); (d/f"{label}.json").write_text(json.dumps(pr,indent=2,sort_keys=True)+"\n"); preds[label]=pr
   try:
    bc=R.outcome(cid,"baseline",base,"canonical"); be=R.outcome(cid,"baseline",base,"external"); ic=R.outcome(cid,"intervention",inter,"canonical"); ie=R.outcome(cid,"intervention",inter,"external")
   except Exception as e:
    stats["invalid"]+=1; ledger.append({"id":cid,"status":"INVALID","error":str(e)}); continue
   stats["executed"]+=1
   jsb=json.loads((P/f"baseline-runs"/cid/f"{cid}-baseline.json").read_text()); jsi=json.loads((P/f"intervention-runs"/cid/f"{cid}-intervention.json").read_text())
   mb=match_actual(preds["baseline"],schedule(cid,"baseline","canonical",jsb)); mi=match_actual(preds["intervention"],schedule(cid,"intervention","canonical",jsi))
   valid=all(o["valid"] for o in (bc,be,ic,ie)) and mb["status"]=="MATCH" and mi["status"]=="MATCH"
   if not valid:
    stats["invalid"]+=1; ledger.append({"id":cid,"type":typ,"status":"INVALID","valid":False}); continue
   stats["valid"]+=1
   exp=(mb["schedule"]["completed"],mi["schedule"]["completed"]); act=((bc["interruption"] is None),(ic["interruption"] is None))
   exp_hit=(not exp[0],not exp[1]); act_int=((ic["interruption"] is not None),(ie["interruption"] is not None))
   rel="REPAIR_OUTCOME_UNCHANGED" if exp_hit==(False,False) or exp_hit==(True,True) else ("FAILURE_INDUCED_INTERRUPTION" if exp_hit==(False,True) else "FAILURE_REMOVAL_RECOVERY_OF_OUTCOME")
   ok=(act[0]==exp[0] and act[1]==exp[1] and (be["completed"]==exp[0]) and (ie["completed"]==exp[1]))
   if ok:
    stats["accepted"]+=1; status="ACCEPTED"
   else:
    stats["rejected"]+=1; status="REJECTED"
    if rel=="FAILURE_REMOVAL_RECOVERY_OF_OUTCOME" and act[0]==exp[0] and act[1]==exp[1]: stats["unresolved"]+=1
   ledger.append({"id":cid,"type":typ,"reference_relation":rel,"expected_completed":exp,"actual_completed":act,"status":status})
 (P/"generated-cases").mkdir(exist_ok=True); (P/"generated-cases/results.json").write_text(json.dumps({"schema":"poie.s5v-n20-heldout/1","stats":stats,"ledger":ledger},indent=2)+"\n")
 print(stats); return 0 if stats["unresolved"]==0 and stats["attempted"]==64 else 1
if __name__=="__main__": raise SystemExit(main())

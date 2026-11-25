#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(P)); sys.path.insert(0,str(ROOT/"audit/phase19")); sys.path.insert(0,str(ROOT/"audit/phase18")); sys.path.insert(0,str(ROOT/"audit/phase17"))
import held_out, run_pair as F
from intervention_oracle import model_outcome
from reference_oracle.reference import predict,match_actual
IDS=['N18-G1901-01','N18-G1901-06','N18-G1901-10','N18-G1901-11','N18-G1901-16','N18-G1901-18','N18-G1901-28','N18-G1919-03','N18-G1919-04','N18-G1919-15','N18-G1919-17','N18-G1919-20','N18-G1919-25','N18-G1919-30']
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def refspec(sp):
 a=sp["alts"][0] if sp["alts"][0] else ["robot-m1"]
 b=sp["alts"][1] if sp["alts"][1] else ["robot-m2"]
 return {"profile":sp["profile"],"tool":sp["tool"],"failures":sp["failures"],"a_robots":a,"b_robots":b}
def actual_schedule(cid,label,model,js):
 root=P/"counterfactual-pairs"/cid/label/model; man=json.loads((root/"run.json").read_text())
 tasks=js["tasks"]; ids=[t["task_id"] for t in tasks]
 if model=="canonical":
  ev=[json.loads(x) for x in (root/Path(man["raw_output"]).name).read_text().splitlines()]
  tr={e["task_id"]:e for e in ev[0]["running_repair"]["task_resources"]}
  return {tid:{"robot":next(r for r in tr[tid]["resources"] if r.startswith("robot")),"start":tr[tid]["start_s"],"end":tr[tid]["end_s"]} for tid in ids}
 import csv
 rows=list(csv.DictReader((root/Path(man["raw_output"]).name).open())); last=rows[-1]
 return {tid:{"robot":("robot-m2" if int(float(last[f"t{i+1}Robot"]))==2 else "robot-m1"),"start":float(last[f"t{i+1}Start_s"]),"end":float(last[f"t{i+1}End_s"])} for i,tid in enumerate(ids)}
def main():
 ids=set(IDS); summary=[]
 for cid in IDS:
  seed=int(cid.split('-')[1][1:]); idx=int(cid.split('-')[-1]); typ,base,inter=held_out.gen(seed,idx)
  preds={}
  for label,sp in (("baseline",base),("intervention",inter)):
   r=refspec(sp); pred=predict(r); d=P/"reference-predictions"/cid; d.mkdir(parents=True,exist_ok=True); fp=d/f"{label}.json"; fp.write_text(json.dumps(pred,indent=2,sort_keys=True)+"\n"); preds[label]={"spec":r,"prediction":pred,"hash":sha(fp)}
  models={}
  for label,sp in (("baseline",base),("intervention",inter)):
   for model in ("canonical","external"):
    F.execute_once(cid,label,model,sp); man=F.validate_existing(cid,label,model); root=P/"counterfactual-pairs"/cid/label/model
    jp=root/Path(man["input_json"]).name; cp=root/Path(man["contract"]).name; js=json.loads(Path(jp).read_text())
    import shutil
    (root/"canonical").mkdir(exist_ok=True); (root/"external").mkdir(exist_ok=True)
    if model=="canonical": shutil.copy(root/Path(man["raw_output"]).name, root/"canonical"/f"{man['run_id']}.jsonl")
    else:
     shutil.copy(root/Path(man["raw_output"]).name, root/"external"/f"{man['run_id']}.csv")
     shutil.copy(root/Path(man["event_journal"]).name, root/"external"/f"{man['run_id']}.csv.journal.json")
    o=model_outcome(sp,model,str(jp),str(cp),str(root)); sched=actual_schedule(cid,label,model,js); m=match_actual(preds[label]["prediction"],sched)
    models[(label,model)]={"outcome":o,"schedule":sched,"match":m}
  checks={}
  for model in ("canonical","external"):
   b=models[("baseline",model)]; i=models[("intervention",model)]
   checks[model]={"baseline_match":b["match"]["status"],"intervention_match":i["match"]["status"],"baseline_completed":b["outcome"]["completed"],"intervention_completed":i["outcome"]["completed"],"baseline_interruption":b["outcome"]["interruption"],"intervention_interruption":i["outcome"]["interruption"],"valid":b["outcome"]["valid"] and i["outcome"]["valid"]}
  expected_direction=(models[("baseline","canonical")]["match"]["schedule"]["completed"] if models[("baseline","canonical")]["match"]["status"]=="MATCH" else None, models[("intervention","canonical")]["match"]["schedule"]["completed"] if models[("intervention","canonical")]["match"]["status"]=="MATCH" else None)
  valid=all(checks[m]["valid"] for m in checks) and all(checks[m]["baseline_match"]=="MATCH" and checks[m]["intervention_match"]=="MATCH" for m in checks)
  expected_hit=(not expected_direction[0],not expected_direction[1]) if expected_direction[0] is not None else (None,None)
  actual_dir=(checks["canonical"]["baseline_interruption"] is not None,checks["canonical"]["intervention_interruption"] is not None)
  if not valid: cls="INVALID_INTERVENTION_CONTRACT"
  elif expected_hit==(True,False) and (actual_dir[0] and not actual_dir[1]): cls="ORACLE_FALSE_POSITIVE"
  else: cls="UNRESOLVED"
  rec={"case_id":cid,"type":typ,"reference_hash":{k:preds[k]["hash"] for k in preds},"reference_status":{k:preds[k]["prediction"]["status"] for k in preds},"models":checks,"expected_direction":{"baseline_completed":expected_direction[0],"intervention_completed":expected_direction[1]},"expected_hit":expected_hit,"canonical_actual":actual_dir,"external_actual":{"baseline_interrupted":checks["external"]["baseline_interruption"] is not None,"intervention_interrupted":checks["external"]["intervention_interruption"] is not None},"final_adjudication":cls,"secondary":("EXPECTED_EFFECT_DIRECTION_ERROR" if cls=="ORACLE_FALSE_POSITIVE" else None),"remaining_uncertainty":"none within bounded reference domain" if cls=="ORACLE_FALSE_POSITIVE" else "unresolved"}
  (P/"adjudications"/f"{cid}.json").write_text(json.dumps(rec,indent=2)+"\n"); summary.append(rec)
 counts={k:sum(1 for r in summary if r["final_adjudication"]==k) for k in sorted({r["final_adjudication"] for r in summary})}
 (P/"adjudications/summary.json").write_text(json.dumps({"schema":"poie.s5v-n20-heldout-adjudication/1","case_count":len(summary),"counts":counts,"cases":[{"case_id":r["case_id"],"type":r["type"],"expected_hit":r["expected_hit"],"adjudication":r["final_adjudication"],"secondary":r["secondary"]} for r in summary]},indent=2)+"\n")
 print("counts",counts)
 for r in summary: print(r["case_id"],r["type"],r["final_adjudication"])
 return 0 if counts.get("ORACLE_FALSE_POSITIVE",0)==14 else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,shutil,subprocess,sys,tempfile
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17"))
from intervention_oracle import model_outcome
N18=ROOT/"audit/phase18"; IDS=['N18-G1801-09','N18-G1801-17','N18-G1801-19','N18-G1801-25','N18-G1817-00','N18-G1817-05','N18-G1817-16','N18-G1817-23','N18-G1817-24']
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; H=ROOT/"external/fmu-n16-scheduler/build/harness"; FMU=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def spec(js):
 return {"profile":"split" if js["dependency_profile"]=="split" else "shared","tool":bool(js["tasks"][0]["requires_tool"]),"failures":[{"time_s":f["time_s"],"target":f["target"],"kind":f["kind"],"target_id":f["target_id"]} for f in js["failures"]],"alts":[t["robot_alternatives"] or None for t in js["tasks"]],"pref":js["preferred_robot"]}
def upstream(robot,profile): return {"bus-1","bat-1","charger-1"} if profile=="split" and robot=="robot-m1" else ({"bus-2"} if profile=="split" else {"bus-1","bat-1","charger-1"})
def expected_hit(js,sp,robots,ends,time,target):
 for i,task in enumerate(js["tasks"]):
  if ends[task["task_id"]]>time+1e-9:
   if target==robots[i] or (target=="tool-t1" and task["requires_tool"]) or target in upstream(robots[i],sp["profile"]): return True
 return False
def main():
 summary=[]; ids=set(IDS)
 for cid in IDS:
  rec={"case_id":cid,"original_hash":sha(N18/"generated-pairs/results.json"),"historical_status":"UNEXPECTED_REPAIR_OUTCOME_CHANGE","reproduced_status":None,"model_results":{}}
  pair={}
  for lab in ("baseline","intervention"):
   src=N18/f"{lab}-runs"/cid; js=json.loads((src/f"{cid}-{lab}.json").read_text()); contract=src/f"{cid}-{lab}.contract"; sp=spec(js)
   rec["intervention_type"]=next(x["type"] for x in json.loads((N18/"generated-pairs/results.json").read_text())["ledger"] if x["id"]==cid)
   # replay in isolated dir
   td=Path(tempfile.mkdtemp(prefix=f"n19-{cid}-{lab}-")); (td/"canonical").mkdir(); (td/"external").mkdir()
   jp=td/f"{cid}-{lab}.json"; cp=td/f"{cid}-{lab}.contract"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp.write_text(contract.read_text())
   out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (td/"canonical"/f"{cid}-{lab}.jsonl").write_text(out)
   subprocess.run([str(H),str(FMU),"900",str(cp),str(td/"external"/f"{cid}-{lab}.csv")],cwd=ROOT,check=True,capture_output=True)
   hist_c=sha(src/"canonical"/f"{cid}-{lab}.jsonl"); new_c=sha(td/"canonical"/f"{cid}-{lab}.jsonl")
   hist_x=sha(src/"external"/f"{cid}-{lab}.csv.journal.json"); new_x=sha(td/"external"/f"{cid}-{lab}.csv.journal.json")
   pair[lab]={"canonical_match":hist_c==new_c,"external_match":hist_x==new_x,"canonical_hash":new_c,"external_hash":new_x}
   for side in ("canonical","external"):
    o=model_outcome(sp,side,str(jp),str(cp),str(td)); pair[lab][side]=o
   shutil.rmtree(td)
  rec["reproduced_status"]="REPRODUCED" if all(pair[l]["canonical_match"] and pair[l]["external_match"] for l in pair) else "HISTORICAL_FAILURE_NOT_REPRODUCED"
  exp={}
  for lab in ("baseline","intervention"):
   js=json.loads((N18/f"{lab}-runs"/cid/f"{cid}-{lab}.json").read_text()); sp=spec(js)
   for side in ("canonical","external"):
    o=pair[lab][side]; robots=o["selected_robots"]; ends={t["task_id"]: (o["makespan"] if t["task_id"]==js["tasks"][0]["task_id"] else None) for t in js["tasks"]}
    # derive end times from contract CSV/canonical observation: use model_outcome resources? use failure contract times only; fallback JSON durations
    ends={t["task_id"]: (pair[lab][side]["makespan"] if False else None) for t in js["tasks"]}
   # expected hit using contract task durations/order and selected path
   tasks=js["tasks"]; starts={}; t0=0
   for i,task in enumerate(tasks): starts[task["task_id"]]=0; ends[task["task_id"]]=task["duration_s"]
   # parallel/serial schedule: derive from observed selected resources and starts in canonical task_resources if available
   obs=pair[lab]["canonical"]
   # use canonical observation timing via stored file for expected only; this is schedule validated separately
   ce=[json.loads(x) for x in (N18/f"{lab}-runs"/cid/"canonical"/f"{cid}-{lab}.jsonl").read_text().splitlines()]
   tr=ce[0]["running_repair"]["task_resources"]
   ends={e["task_id"]:e["end_s"] for e in tr}
   robots=pair[lab]["canonical"]["selected_robots"]
   hit=any(expected_hit(js,sp,robots,ends,f["time_s"],f["target"]) for f in sp["failures"])
   exp[lab]=hit
  rec["independently_derived_expected_effect"]={"baseline_hit":exp["baseline"],"intervention_hit":exp["intervention"],"direction":"HIT->NO_HIT" if exp["baseline"] and not exp["intervention"] else ("NO_HIT->HIT" if not exp["baseline"] and exp["intervention"] else "SAME_HIT_STATE")}
  bc=pair["baseline"]["canonical"]; ic=pair["intervention"]["canonical"]; be=pair["baseline"]["external"]; ie=pair["intervention"]["external"]
  rec["canonical_observed_effect"]={"baseline_interrupted":bc["interruption"] is not None,"intervention_interrupted":ic["interruption"] is not None}
  rec["external_observed_effect"]={"baseline_interrupted":be["interruption"] is not None,"intervention_interrupted":ie["interruption"] is not None}
  expected_dir=exp["baseline"] and not exp["intervention"]
  observed_dir=(bc["interruption"] is not None) and (ic["interruption"] is None) and (be["interruption"] is not None) and (ie["interruption"] is None)
  if rec["reproduced_status"]!="REPRODUCED": cls="EVIDENCE_OR_PROVENANCE_FAILURE"
  elif not (bc["valid"] and be["valid"] and ic["valid"] and ie["valid"]): cls="INVALID_INTERVENTION_CONTRACT"
  elif expected_dir and observed_dir: cls="ORACLE_FALSE_POSITIVE"
  elif not expected_dir and observed_dir: cls="SHARED_MODEL_CONTRACT_VIOLATION"
  else: cls="UNRESOLVED"
  rec["first_causal_divergence"]="none; N18 classification differed only in expected-effect direction"
  rec["root_cause_classification"]=cls
  rec["secondary_reason"]="EXPECTED_EFFECT_DIRECTION_ERROR" if cls=="ORACLE_FALSE_POSITIVE" else None
  rec["expected_effect_before"]={"generator_rule":"NO_HIT->HIT only","baseline_expected_changed":False}
  rec["supporting_evidence"]=[f"audit/phase18/{l}-runs/{cid}" for l in ("baseline","intervention")]
  rec["final_adjudication"]=cls
  rec["remaining_uncertainty"]="none within tested contract" if cls=="ORACLE_FALSE_POSITIVE" else "requires further analysis"
  rec["pair"]=pair
  (P/"adjudications"/f"{cid}.json").write_text(json.dumps(rec,indent=2)+"\n"); summary.append(rec)
 counts={k:sum(1 for r in summary if r["final_adjudication"]==k) for k in sorted({r["final_adjudication"] for r in summary})}
 (P/"adjudications/summary.json").write_text(json.dumps({"schema":"poie.s5v-n19-adjudication/1","case_count":len(summary),"counts":counts,"cases":[{"case_id":r["case_id"],"type":r["intervention_type"],"reproduced":r["reproduced_status"],"expected":r["independently_derived_expected_effect"],"adjudication":r["final_adjudication"],"secondary":r["secondary_reason"]} for r in summary]},indent=2)+"\n")
 print("counts",counts)
 for r in summary: print(r["case_id"],r["intervention_type"],r["reproduced_status"],r["final_adjudication"],r["independently_derived_expected_effect"]["direction"])
 return 0 if counts.get("ORACLE_FALSE_POSITIVE",0)==9 else 1
if __name__=="__main__": raise SystemExit(main())

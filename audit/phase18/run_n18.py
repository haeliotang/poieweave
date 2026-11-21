#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17"))
from intervention_oracle import model_outcome
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; H=ROOT/"external/fmu-n16-scheduler/build/harness"; FMU=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"
T={0:"robot-m1",1:"robot-m2",3:"bus-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}; K={0:1,1:1,7:2}
def spec(profile="split",tool=False,failures=(),alts=(None,None),pref=0,order=("A","B"),b_mask=2,a_mask=1):
 return {"profile":profile,"tool":tool,"failures":[{"time_s":t,"target":T[a],"kind":K.get(a,3),"target_id":a} for t,a in failures],"alts":list(alts),"pref":pref,"order":list(order),"b_mask":b_mask,"a_mask":a_mask}
def js_for(cid,label,sp):
 tasks={ "A":{"task_id":"A","duration_s":7200,"priority":1,"robot_mask":sp.get("a_mask",1),"requires_tool":sp["tool"],"robot_alternatives":[]},
         "B":{"task_id":"B","duration_s":10800,"priority":2,"robot_mask":sp.get("b_mask",2),"requires_tool":sp["tool"],"robot_alternatives":[]}}
 if sp["alts"][0]: tasks["A"].update(robot_mask=3,robot_alternatives=sp["alts"][0])
 if sp["alts"][1]: tasks["B"].update(robot_mask=3,robot_alternatives=sp["alts"][1])
 return {"group_id":f"{cid}-{label}","restoration_target":"a-lathe","horizon_s":72000,"robot_count":2,"release_mode":"task_release","dependency_profile":"split" if sp["profile"]=="split" else "default","preferred_robot":sp["pref"],"tasks":[tasks[x] for x in sp["order"]],"failures":[dict(f,failure_id=i) for i,f in enumerate(sp["failures"])]}
def write_contract(js,path):
 t=js["tasks"]; b={"taskCount":2,"t1Duration_s":t[0]["duration_s"],"t1Priority":1,"t1RobotMask":t[0]["robot_mask"],"t1RequiresTool":int(t[0]["requires_tool"]),"t2Duration_s":t[1]["duration_s"],"t2Priority":2,"t2RobotMask":t[1]["robot_mask"],"t2RequiresTool":int(t[1]["requires_tool"]),"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":2,"horizon_s":js["horizon_s"],"preferredRobot":js["preferred_robot"],"robot1DependencyMask":28,"robot2DependencyMask":64 if js["dependency_profile"]=="split" else 28,"failureCount":len(js["failures"]),"releaseMode":1}
 for i,f in enumerate(js["failures"]):
  pre="failure" if i==0 else f"f{i+1}"; b[f"{pre}Time_s"]=f["time_s"]; b[f"{pre}Kind"]=f["kind"]
  if f["kind"]==3: b[f"{pre}AssetId"]=f["target_id"]
  elif f["kind"]==1: b[f"{pre}Index"]=f["target_id"]+1
 Path(path).write_text("\n".join(f"{k}={v}" for k,v in b.items())+"\n")
def exec_spec(cid,label,sp):
 root=P/("baseline-runs" if label=="baseline" else "intervention-runs")/cid; (root/"canonical").mkdir(parents=True,exist_ok=True); (root/"external").mkdir(parents=True,exist_ok=True)
 js=js_for(cid,label,sp); jp=root/f"{cid}-{label}.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=root/f"{cid}-{label}.contract"; write_contract(js,cp)
 out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (root/"canonical"/f"{cid}-{label}.jsonl").write_text(out)
 subprocess.run([str(H),str(FMU),"900",str(cp),str(root/"external"/f"{cid}-{label}.csv")],cwd=ROOT,check=True,capture_output=True)
 return root,jp,cp
def outcome(cid,label,sp,side):
 root,jp,cp=exec_spec(cid,label,sp); return model_outcome(sp,side,str(jp),str(cp),str(root))
def classify(bc,be,ic,ie,exp):
 valid=all(o["valid"] for o in (bc,be,ic,ie))
 if not valid: return "INVALID_EXECUTION"
 oc=bc["completed"]!=ic["completed"] or (bc["interruption"] is None)!=(ic["interruption"] is None)
 oe=be["completed"]!=ie["completed"] or (be["interruption"] is None)!=(ie["interruption"] is None)
 outcome_changed=oc or oe
 failure_changed=tuple(sorted(bc["failures"]))!=tuple(sorted(ic["failures"])) or tuple(sorted(be["failures"]))!=tuple(sorted(ie["failures"]))
 schedule_changed=tuple(bc["selected_robots"])!=tuple(ic["selected_robots"]) or tuple(be["selected_robots"])!=tuple(ie["selected_robots"]) or abs(bc["makespan"]-ic["makespan"])>1e-9 or abs(be["makespan"]-ie["makespan"])>1e-9
 policy=any(bc["selected_robots"][i]!=be["selected_robots"][i] for i in range(2)) or any(ic["selected_robots"][i]!=ie["selected_robots"][i] for i in range(2))
 if exp.get("outcome") is True:
  rel="CHANGED_REPAIR_OUTCOME_EXPLAINED" if outcome_changed else "EXPECTED_EFFECT_MISSING"
 elif exp.get("outcome") is False:
  if outcome_changed: rel="UNEXPECTED_REPAIR_OUTCOME_CHANGE"
  elif failure_changed: rel="DIFFERENT_FAILURE_HISTORY_SAME_REPAIR_OUTCOME"
  elif schedule_changed: rel="CHANGED_SCHEDULE_EXPLAINED"
  else: rel="SAME_REPAIR_OUTCOME_VALID"
 else: rel="UNEXPLAINED_CAUSAL_DIVERGENCE"
 if policy and rel not in ("INVALID_EXECUTION","EXPECTED_EFFECT_MISSING","UNEXPECTED_REPAIR_OUTCOME_CHANGE"): rel="VALID_POLICY_DEPENDENT_DIVERGENCE"
 return rel
CASES=[
 ("N18-00","FAILURE_INSERTION",spec(failures=()),spec(failures=((3600,5),)),{"outcome":False}),
 ("N18-01","FAILURE_INSERTION",spec(failures=()),spec(failures=((3600,3),)),{"outcome":True}),
 ("N18-02","FAILURE_TIME_CHANGE",spec(failures=((7199,3),)),spec(failures=((7201,3),)),{"outcome":True}),
 ("N18-03","DEPENDENCY_EDGE_CHANGE",spec(profile="split",failures=((7201,3),)),spec(profile="shared",failures=((7201,3),)),{"outcome":True}),
 ("N18-04","RESOURCE_ELIGIBILITY_CHANGE",spec(alts=(("robot-m1",),None)),spec(alts=(("robot-m2",),None),pref=2),{"outcome":False,"schedule":True}),
 ("N18-05","RESOURCE_ELIGIBILITY_CHANGE",spec(alts=(("robot-m2",),None),order=("A","B")),spec(alts=(("robot-m2",),None),order=("A","B"),failures=((7201,6),)),{"outcome":True}),
 ("N18-06","RESOURCE_ELIGIBILITY_CHANGE",spec(alts=(("robot-m2",),None),order=("A","B")),spec(alts=(("robot-m2",),None),order=("A","B"),failures=((7201,6),)),{"outcome":True}),
 ("N18-07","RESOURCE_ELIGIBILITY_CHANGE",spec(tool=True,alts=(("robot-m1",),None)),spec(tool=True,alts=(("robot-m2",),None),pref=2,failures=((3600,7),)),{"outcome":True}),
 ("N18-08","FAILURE_INSERTION",spec(),spec(failures=((10800,6),)),{"outcome":False}),
 ("N18-09","FAILURE_INSERTION",spec(),spec(failures=((72000,6),)),{"outcome":False}),
 ("N18-10","FAILURE_INSERTION",spec(),spec(failures=((3600,3),(3600,6))),{"outcome":True}),
 ("N18-11","FAILURE_INSERTION",spec(profile="split"),spec(profile="split",failures=((7201,3),)),{"outcome":False}),
 ("N18-15","OR_SPLIT_WITNESS",spec(alts=(("robot-m1","robot-m2"),None),order=("B","A"),b_mask=1,failures=((7199,6),)),spec(alts=(("robot-m1","robot-m2"),None),order=("B","A"),b_mask=1,failures=((7201,6),)),{"outcome":True}),
]
def main():
 results=[]
 for cid,itype,base,interv,exp in CASES:
  bc=outcome(cid,"baseline",base,"canonical"); be=outcome(cid,"baseline",base,"external"); ic=outcome(cid,"intervention",interv,"canonical"); ie=outcome(cid,"intervention",interv,"external")
  rel=classify(bc,be,ic,ie,exp); ok=rel in ("CHANGED_REPAIR_OUTCOME_EXPLAINED","DIFFERENT_FAILURE_HISTORY_SAME_REPAIR_OUTCOME","CHANGED_SCHEDULE_EXPLAINED","SAME_REPAIR_OUTCOME_VALID","VALID_POLICY_DEPENDENT_DIVERGENCE")
  results.append({"intervention_id":cid,"type":itype,"expected":exp,"valid":all(o["valid"] for o in (bc,be,ic,ie)),"relation":rel,"baseline":{"canonical":bc,"external":be},"intervention":{"canonical":ic,"external":ie},"status":"ACCEPTED" if ok else "FAIL"})
 (P/"causal-comparisons").mkdir(exist_ok=True); (P/"causal-comparisons/fixed.json").write_text(json.dumps({"schema":"poie.s5v-n18-fixed/1","pairs":results},indent=2)+"\n")
 for r in results: print(r["intervention_id"],r["relation"],r["status"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

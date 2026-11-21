#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
sys.path.insert(0,str(P)); from intervention_oracle import model_outcome,relation
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n16-scheduler/build/harness"; FMU=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"
T={0:"robot-m1",1:"robot-m2",3:"bus-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}; K={0:1,1:1,7:2}
def spec(profile="split",tool=False,failures=(),alts=None,pref=0):
 return {"profile":profile,"tool":tool,"failures":[{"time_s":t,"target":T[a],"kind":K.get(a,3),"target_id":a} for t,a in failures],"alts":alts or [None,None],"pref":pref}
def write_contract(js,path):
 t=js["tasks"]; b={"taskCount":2,"t1Duration_s":t[0]["duration_s"],"t1Priority":1,"t1RobotMask":t[0]["robot_mask"],"t1RequiresTool":int(t[0]["requires_tool"]),"t2Duration_s":t[1]["duration_s"],"t2Priority":2,"t2RobotMask":t[1]["robot_mask"],"t2RequiresTool":int(t[1]["requires_tool"]),"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":2,"horizon_s":js["horizon_s"],"preferredRobot":js["preferred_robot"],"robot1DependencyMask":28,"robot2DependencyMask":64 if js["dependency_profile"]=="split" else 28,"failureCount":len(js["failures"]),"releaseMode":1}
 for i,f in enumerate(js["failures"]):
  pre="failure" if i==0 else f"f{i+1}"; b[f"{pre}Time_s"]=f["time_s"]; b[f"{pre}Kind"]=f["kind"]
  if f["kind"]==3: b[f"{pre}AssetId"]=f["target_id"]
  elif f["kind"]==1: b[f"{pre}Index"]=f["target_id"]+1
 Path(path).write_text("\n".join(f"{k}={v}" for k,v in b.items())+"\n")
def make_js(pair,label,sp):
 t=[{"task_id":"A","duration_s":7200,"priority":1,"robot_mask":1,"requires_tool":sp["tool"],"robot_alternatives":[]},
    {"task_id":"B","duration_s":10800,"priority":2,"robot_mask":2,"requires_tool":sp["tool"],"robot_alternatives":[]}]
 if sp["alts"][0]: t[0]["robot_mask"]=3; t[0]["robot_alternatives"]=sp["alts"][0]
 if sp["alts"][1]: t[1]["robot_mask"]=3; t[1]["robot_alternatives"]=sp["alts"][1]
 return {"group_id":f"{pair}-{label}","restoration_target":"a-lathe","horizon_s":72000,"robot_count":2,"release_mode":"task_release","dependency_profile":"split" if sp["profile"]=="split" else "default","preferred_robot":sp["pref"],"tasks":t,
  "failures":[{"failure_id":i,"time_s":f["time_s"],"target":f["target"],"kind":f["kind"],"target_id":f["target_id"]} for i,f in enumerate(sp["failures"])]}
def execute(pair,label,sp):
 root=P/("baseline-runs" if label=="baseline" else "intervention-runs")/pair; (root/"canonical").mkdir(parents=True,exist_ok=True); (root/"external").mkdir(parents=True,exist_ok=True)
 js=make_js(pair,label,sp); jp=root/f"{pair}-{label}.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=root/f"{pair}-{label}.contract"; write_contract(js,cp)
 out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (root/"canonical"/f"{pair}-{label}.jsonl").write_text(out)
 subprocess.run([str(HARNESS),str(FMU),"900",str(cp),str(root/"external"/f"{pair}-{label}.csv")],cwd=ROOT,check=True,capture_output=True)
 return {"root":str(root),"json":str(jp),"contract":str(cp)}
FIXED=[
 ("N17-C1","FAILURE_TARGET_CHANGE",spec("split",False,[(3600,5)]),spec("split",False,[(3600,3)]),False),
 ("N17-C2","FAILURE_TIME_CHANGE",spec("split",False,[(7199,3)]),spec("split",False,[(7201,3)]),False),
 ("N17-C3","DEPENDENCY_EDGE_CHANGE",spec("split",False,[(7201,3)]),spec("shared",False,[(7201,3)]),False),
 ("N17-C4","RESOURCE_ELIGIBILITY_CHANGE",spec("split",False,[], [["robot-m1"],None],0),spec("split",False,[], [["robot-m2"],None],2),False),
 ("N17-C7","FAILURE_INSERTION",spec("split",False,[]),spec("split",False,[(7201,3)]),False),
 ("N17-C8a","FAILURE_TIME_CHANGE",spec("split",False,[(7199,3)]),spec("split",False,[(7200,3)]),False),
 ("N17-C8b","FAILURE_TIME_CHANGE",spec("split",False,[(7200,3)]),spec("split",False,[(7201,3)]),False),
 ("N17-C9","FAILURE_TIME_CHANGE",spec("shared",False,[(7199,3)]),spec("shared",False,[(7201,3)]),False),
 ("N17-C6","RESOURCE_ELIGIBILITY_CHANGE",spec("split",True,[(3600,7)],[["robot-m1"],None],0),spec("split",True,[(3600,7)],[["robot-m2"],None],2),False),
 ("N17-C10","FAILURE_TIME_CHANGE",spec("split",False,[(10799,6)]),spec("split",False,[(10801,6)]),False),
]
def main():
 results=[]; outcomes={}
 for pair,itype,base,interv,policy in FIXED:
  b=execute(pair,"baseline",base); i=execute(pair,"intervention",interv)
  bc=model_outcome(base,"canonical",b["json"],b["contract"],b["root"]); be=model_outcome(base,"external",b["json"],b["contract"],b["root"])
  ic=model_outcome(interv,"canonical",i["json"],i["contract"],i["root"]); ie=model_outcome(interv,"external",i["json"],i["contract"],i["root"])
  pol=policy or any(bc["selected_robots"][k]!=be["selected_robots"][k] or ic["selected_robots"][k]!=ie["selected_robots"][k] for k in range(2))
  rel,detail=relation(bc,be,ic,ie,pol)
  results.append({"intervention_id":pair,"type":itype,"policy_related":pol,"changed":{"profile":base["profile"]!=interv["profile"],"failures":base["failures"]!=interv["failures"],"alts":base["alts"]!=interv["alts"],"pref":base["pref"]!=interv["pref"],"tool":base["tool"]!=interv["tool"]},"unchanged":{"tasks":True,"horizon":True},"baseline":{"canonical":bc,"external":be},"intervention":{"canonical":ic,"external":ie},"relation":rel,"relation_detail":detail,"status":"ACCEPTED" if rel in ("SAME_OUTCOME_VALID","CHANGED_OUTCOME_EXPLAINED","VALID_POLICY_DEPENDENT_DIVERGENCE") else "FAIL"})
 (P/"oracle-results").mkdir(exist_ok=True); (P/"oracle-results/fixed-pairs.json").write_text(json.dumps({"schema":"poie.s5v-n17-interventions/1","pairs":results},indent=2)+"\n")
 for r in results: print(r["intervention_id"],r["relation"],r["status"],[r["baseline"]["canonical"]["findings"],r["baseline"]["external"]["findings"],r["intervention"]["canonical"]["findings"],r["intervention"]["external"]["findings"]])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

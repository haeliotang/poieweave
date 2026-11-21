#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; D=P/"alternative-resource"; D.mkdir(parents=True,exist_ok=True)
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n14-scheduler/build/harness"; FMU=ROOT/"external/fmu-n14-scheduler/poie-n14-scheduler.fmu"
ASSETS={0:"robot-m1",1:"robot-m2",5:"sensor-x",6:"bus-2",3:"bus-1",7:"tool-t1"}
CASES={
 "OR-01":({"A":["robot-m1","robot-m2"],"B":["robot-m1","robot-m2"]},True,0,[]),
 "OR-02":({"A":["robot-m1","robot-m2"],"B":["robot-m1","robot-m2"]},True,0,[]),
 "OR-03":({"A":["robot-m1","robot-m2"],"B":["robot-m1","robot-m2"]},True,2,[]),
 "OR-04":({"A":["robot-m1","robot-m2"],"B":["robot-m1","robot-m2"]},True,2,[(3600,0)]),
 "OR-05":({"A":["robot-m1","robot-m2"],"B":["robot-m1","robot-m2"]},True,0,[(3600,7)]),
 "OR-06":({"A":["robot-m1"],"B":["robot-m1","robot-m2"]},False,0,[]),
}
def js_for(cid,alts,tool,preferred,failures):
 tasks=[{"task_id":"A","duration_s":7200,"priority":1,"robot_mask":3,"requires_tool":tool,"robot_alternatives":alts["A"]},
        {"task_id":"B","duration_s":10800,"priority":2,"robot_mask":3,"requires_tool":False,"robot_alternatives":alts["B"]}]
 return {"group_id":cid,"restoration_target":"a-lathe","horizon_s":72000,"robot_count":2,"preferred_robot":preferred,"tasks":tasks,
         "failures":[{"failure_id":i,"time_s":t,"target":ASSETS[a],"kind":1 if a in (0,1) else (2 if a==7 else 3),"target_id":a} for i,(t,a) in enumerate(failures)]}
def key(js,cid):
 base={"taskCount":2,"t1Duration_s":7200,"t1Priority":1,"t1RobotMask":3,"t1RequiresTool":int(js["tasks"][0]["requires_tool"]),"t2Duration_s":10800,"t2Priority":2,"t2RobotMask":3,"t2RequiresTool":0,"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":2,"horizon_s":72000,"preferredRobot":js["preferred_robot"],"robot1DependencyMask":28,"robot2DependencyMask":28,"failureCount":len(js["failures"])}
 for i,f in enumerate(js["failures"]):
  pre="failure" if i==0 else f"f{i+1}"; base[f"{pre}Time_s"]=f["time_s"]
  if f["kind"]==1: base[f"{pre}Kind"]=1; base[f"{pre}Index"]=f["target_id"]+1; base[f"{pre}AssetId"]=-1
  elif f["kind"]==2: base[f"{pre}Kind"]=2; base[f"{pre}AssetId"]=-1
  else: base[f"{pre}Kind"]=3; base[f"{pre}AssetId"]=f["target_id"]
 return "\n".join(f"{k}={v}" for k,v in base.items())+"\n"
def main():
 results=[]
 for cid,(alts,tool,pref,failures) in CASES.items():
  js=js_for(cid,alts,tool,pref,failures); jp=D/f"{cid}.json"; cp=D/f"{cid}.contract"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp.write_text(key(js,cid))
  out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (D/f"{cid}.jsonl").write_text(out)
  subprocess.run([str(HARNESS),str(FMU),"900",str(cp),str(D/f"{cid}.csv")],cwd=ROOT,check=True,capture_output=True)
  ce=[json.loads(x) for x in out.splitlines()]; last=list(csv.DictReader((D/f"{cid}.csv").open()))[-1]
  tr=ce[0]["running_repair"]["task_resources"]; problems=[]
  for i,t in enumerate(js["tasks"]):
   got=next(x["resources"] for x in tr if x["task_id"]==t["task_id"] and "robot" in "".join(x["resources"]))
   robot=[r for r in got if r.startswith("robot")][0]
   if robot not in t["robot_alternatives"]: problems.append(f"TASK_RESOURCE_{i}")
   er=ASSETS[int(float(last[f"t{i+1}Robot"]))-1]
   if er not in t["robot_alternatives"]: problems.append(f"EXTERNAL_TASK_RESOURCE_{i}")
  cint=any(e["event_kind"]=="RepairInterrupted" for e in ce)
  xint=any(e["kind"]=="RepairInterrupted" for e in json.loads((Path(str(D/f"{cid}.csv")+".journal.json")).read_text())["events"])
  classification="SAME_DECISION"
  erobots=[ASSETS[int(float(last[f"t{i+1}Robot"]))-1] for i in range(2)]
  crobots=[next(r for r in x["resources"] if r.startswith("robot")) for x in tr]
  if crobots!=erobots:
   classification="DIFFERENT_POLICY_VALID_DECISIONS"
  if cint!=xint:
   only_c=set(crobots)-set(erobots); only_e=set(erobots)-set(crobots)
   fail_targets={ASSETS[f[1]] for f in failures}
   if classification=="DIFFERENT_POLICY_VALID_DECISIONS" and cint and (fail_targets & only_c) and not (fail_targets & only_e):
    classification="VALID_POLICY_DEPENDENT_DIVERGENCE"
   else: problems.append("INTERRUPTION")
  results.append({"case":cid,"canonical_robots":crobots,"external_robots":erobots,"canonical_interrupted":cint,"external_interrupted":xint,"classification":classification,"problems":problems,"status":"ACCEPTED" if not problems else "FAIL"})
 (D/"results.json").write_text(json.dumps({"schema":"poie.s5v-n14rh-or/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["canonical_robots"],r["external_robots"],r["classification"],r["problems"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

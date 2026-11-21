#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,io,json,subprocess
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
def key(js):
 base={"taskCount":2,"t1Duration_s":7200,"t1Priority":1,"t1RobotMask":3,"t1RequiresTool":int(js["tasks"][0]["requires_tool"]),"t2Duration_s":10800,"t2Priority":2,"t2RobotMask":3,"t2RequiresTool":0,"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":2,"horizon_s":72000,"preferredRobot":js["preferred_robot"],"robot1DependencyMask":28,"robot2DependencyMask":28,"failureCount":len(js["failures"])}
 for i,f in enumerate(js["failures"]):
  pre="failure" if i==0 else f"f{i+1}"; base[f"{pre}Time_s"]=f["time_s"]
  if f["kind"]==1: base[f"{pre}Kind"]=1; base[f"{pre}Index"]=f["target_id"]+1; base[f"{pre}AssetId"]=-1
  elif f["kind"]==2: base[f"{pre}Kind"]=2; base[f"{pre}AssetId"]=-1
  else: base[f"{pre}Kind"]=3; base[f"{pre}AssetId"]=f["target_id"]
 return "\n".join(f"{k}={v}" for k,v in base.items())+"\n"
def compare(cid,js,ce=None,rows=None,journal=None):
 if ce is None: ce=[json.loads(x) for x in (D/f"{cid}.jsonl").read_text().splitlines()]
 if rows is None: rows=list(csv.DictReader((D/f"{cid}.csv").open()))
 if journal is None: journal=json.loads((Path(str(D/f"{cid}.csv")+".journal.json")).read_text())["events"]
 tasks=js["tasks"]; findings=[]; tr=ce[0]["running_repair"]["task_resources"]
 last=rows[-1]
 if len(tr)!=2: findings.append("TASK_COUNT")
 robots=[]; starts=[]; ends=[]
 for i,task in enumerate(tasks):
  tid=task["task_id"]; entry=next((x for x in tr if x.get("task_id")==tid),None)
  if entry is None: findings.append(f"TASK_MISSING_{tid}"); continue
  got=set(entry["resources"]); crobot=next((r for r in got if r.startswith("robot")),None)
  if crobot not in task["robot_alternatives"]: findings.append(f"RESOURCE_ELIGIBILITY_VIOLATION_{tid}")
  if bool(task["requires_tool"]) != ("tool-t1" in got): findings.append(f"REQUIRED_TOOL_MISSING_{tid}")
  erobot=ASSETS[int(float(last[f"t{i+1}Robot"]))-1]
  if erobot not in task["robot_alternatives"]: findings.append(f"EXTERNAL_RESOURCE_ELIGIBILITY_{tid}")
  if abs(entry["start_s"]-float(last[f"t{i+1}Start_s"]))>1e-9: findings.append(f"TASK_START_MISMATCH_{tid}")
  if abs(entry["end_s"]-float(last[f"t{i+1}End_s"]))>1e-9: findings.append(f"TASK_END_MISMATCH_{tid}")
  etool=bool(int(float(last["toolTaskMask"])) & (1<<i))
  if bool(task["requires_tool"]) != etool: findings.append(f"EXTERNAL_REQUIRED_TOOL_MISSING_{tid}")
  robots.append(crobot); starts.append(entry["start_s"]); ends.append(entry["end_s"])
 makespan=max(ends) if ends else 0.0
 if abs(makespan-float(last["makespan_s"]))>1e-9: findings.append("GROUP_MAKESPAN_MISMATCH")
 # per-task legality and contention
 for i in range(len(robots)):
  for j in range(i+1,len(robots)):
   if robots[i]==robots[j] and starts[j] < ends[i]-1e-9: findings.append("RESOURCE_CONTENTION_VIOLATION")
 # failures and sensitivity per model using its own selected resources
 cf=[e for e in ce if e["event_kind"]=="AssetFailure"]
 xf=[e for e in journal if e["kind"]=="AssetFailure"]
 if len(cf)!=len(xf): findings.append("EVENT_COUNT")
 for c,x in zip(cf,xf):
  if abs(c["at_s"]-x["at"])>1e-9: findings.append("EVENT_TIME")
  if c["target_asset"]!=x["target"]: findings.append("EVENT_TARGET")
 # expected interruption from each model's own selected resources
 def expected_hit(target, time, robot_list, end_list):
  for i,task in enumerate(tasks):
   if time < end_list[i]-1e-9:
    if target==robot_list[i]: return True
    if target=="tool-t1" and task["requires_tool"]: return True
    if target in ("bus-1","bus-2") and target=="bus-1": return True
  return False
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in journal)
 c_expected=any(expected_hit(x["target"],x["at"],robots,ends) for x in xf)
 er=[ASSETS[int(float(last[f"t{i+1}Robot"]))-1] for i in range(2)]; ee=[float(last[f"t{i+1}End_s"]) for i in range(2)]
 x_expected=any(expected_hit(x["target"],x["at"],er,ee) for x in xf)
 if c_int!=c_expected: findings.append("CANONICAL_FAILURE_SENSITIVITY")
 if x_int!=x_expected: findings.append("EXTERNAL_FAILURE_SENSITIVITY")
 # group completion/restoration
 c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in journal)
 if c_comp!=x_comp: findings.append("GROUP_COMMIT_MISMATCH")
 if any(e["kind"]=="RestorationCommitted" and e["at"] < makespan-1e-9 for e in journal): findings.append("PREMATURE_GROUP_COMMIT")
 if x_int and any(e["kind"]=="RestorationCommitted" for e in journal): findings.append("INTERRUPTED_RESTORED")
 classification="SAME_DECISION"
 crobots=[next((r for r in x["resources"] if r.startswith("robot")),None) for x in tr]
 erobots=[ASSETS[int(float(last[f"t{i+1}Robot"]))-1] for i in range(2)]
 if crobots!=erobots:
  classification="DIFFERENT_POLICY_VALID_DECISIONS"
  findings=[f for f in findings if not (f.startswith("TASK_START_MISMATCH") or f.startswith("TASK_END_MISMATCH") or f in ("GROUP_MAKESPAN_MISMATCH","GROUP_COMMIT_MISMATCH","INTERRUPTED_RESTORED","PREMATURE_GROUP_COMMIT"))]
 if c_int!=x_int:
  only_c=set(crobots)-set(erobots); only_e=set(erobots)-set(crobots); ft={ASSETS[f["target_id"]] for f in js["failures"]}
  if classification=="DIFFERENT_POLICY_VALID_DECISIONS" and c_int and (ft&only_c) and not (ft&only_e): classification="VALID_POLICY_DEPENDENT_DIVERGENCE"
  else: findings.append("INTERRUPTION")
 return {"case":cid,"canonical_robots":crobots,"external_robots":erobots,"canonical_interrupted":c_int,"external_interrupted":x_int,"classification":classification,"findings":findings,"status":"ACCEPTED" if not findings else "FAIL"}
def main():
 results=[]
 for cid,(alts,tool,pref,failures) in CASES.items():
  js=js_for(cid,alts,tool,pref,failures); (D/f"{cid}.json").write_text(json.dumps(js,indent=2)+"\n"); (D/f"{cid}.contract").write_text(key(js))
  out=subprocess.run([str(CANON),str(D/f"{cid}.json")],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (D/f"{cid}.jsonl").write_text(out)
  subprocess.run([str(HARNESS),str(FMU),"900",str(D/f"{cid}.contract"),str(D/f"{cid}.csv")],cwd=ROOT,check=True,capture_output=True)
  results.append(compare(cid,js))
 (D/"results.json").write_text(json.dumps({"schema":"poie.s5v-n14rh-or/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["classification"],r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

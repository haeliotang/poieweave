#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n14-scheduler/build/harness"; FMU=ROOT/"external/fmu-n14-scheduler/poie-n14-scheduler.fmu"
ASSETS={0:"robot-m1",1:"robot-m2",2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}
CASES=[
 ("N14R-00",[(7200,1,False),(10800,2,False)],[],2),
 ("N14R-01",[(7200,1,False),(10800,1,False)],[],1),
 ("N14R-02",[(7200,1,False),(10800,2,False)],[(3600,5),(5400,6)],2),
 ("N14R-03",[(7200,1,False),(10800,2,False)],[(3600,5),(3600,6)],2),
 ("N14R-04",[(7200,1,False),(10800,2,False)],[(3600,0)],2),
 ("N14R-05",[(7200,1,True),(10800,2,False)],[(3600,7)],2),
 ("N14R-06",[(7200,1,False),(10800,2,False)],[(3600,3)],2),
 ("N14R-07",[(7200,1,True),(10800,2,False)],[(3600,5),(3600,6),(5400,0),(5400,7)],2),
 ("N14R-08",[(7200,1,False),(10800,2,False)],[(7201,0)],2),
 ("N14R-09",[(7200,1,False),(10800,2,False)],[(10800,0)],2),
]
def write_contract(cid,tasks,failures,robots):
 js={"group_id":cid,"restoration_target":"a-lathe","horizon_s":72000,"robot_count":robots,
     "tasks":[{"task_id":chr(65+i),"duration_s":d,"priority":i+1,"robot_mask":m,"requires_tool":tool} for i,(d,m,tool) in enumerate(tasks)],
     "failures":[{"failure_id":i,"time_s":t,"target":ASSETS[a],"kind":1 if a in (0,1) else (2 if a==7 else 3),"target_id":a} for i,(t,a) in enumerate(failures)]}
 jp=P/"contracts"/f"{cid}.json"; jp.write_text(json.dumps(js,indent=2)+"\n")
 cp=P/"contracts"/f"{cid}.contract"; base={"taskCount":2,"t1Duration_s":tasks[0][0],"t1Priority":1,"t1RobotMask":tasks[0][1],"t1RequiresTool":int(tasks[0][2]),"t2Duration_s":tasks[1][0],"t2Priority":2,"t2RobotMask":tasks[1][1],"t2RequiresTool":int(tasks[1][2]),"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":robots,"horizon_s":72000,"preferredRobot":0,"robot1DependencyMask":28,"robot2DependencyMask":28,"failureCount":len(failures)}
 for i,(t,a) in enumerate(failures):
  pre="failure" if i==0 else f"f{i+1}"; base[f"{pre}Time_s"]=t
  if a in (0,1): base[f"{pre}Kind"]=1; base[f"{pre}Index"]=a+1; base[f"{pre}AssetId"]=-1
  elif a==7: base[f"{pre}Kind"]=2; base[f"{pre}AssetId"]=-1
  else: base[f"{pre}Kind"]=3; base[f"{pre}AssetId"]=a
 cp.write_text("\n".join(f"{k}={v}" for k,v in base.items())+"\n")
 return js
def compare(cid,tasks,failures):
 ce=[json.loads(x) for x in (P/"canonical"/f"{cid}.jsonl").read_text().splitlines()]
 ext=list(csv.DictReader((P/"external"/f"{cid}.csv").open())); last=ext[-1]
 j=json.loads((P/"external"/f"{cid}.csv.journal.json").read_text()); ev=j["events"]
 problems=[]
 # task schedules and resources
 first=ce[0]["running_repair"]; ct=first["task_resources"]
 if len(ct)!=2: problems.append("TASK_COUNT")
 for i,entry in enumerate(ct[:2]):
  if int(float(last[f"t{i+1}Start_s"]))!=int(entry["start_s"]): problems.append(f"TASK_START_{i}")
  if int(float(last[f"t{i+1}End_s"]))!=int(entry["end_s"]): problems.append(f"TASK_END_{i}")
  cr=entry["resources"][0]; er=ASSETS[int(float(last[f"t{i+1}Robot"]))-1]
  if cr!=er: problems.append(f"TASK_RESOURCE_{i}")
 makespan=max(entry["end_s"] for entry in ct)
 if abs(makespan-float(last["makespan_s"]))>1e-9: problems.append("GROUP_MAKESPAN")
 # failures
 xf=[e for e in ev if e["kind"]=="AssetFailure"]
 cf=[e for e in ce if e["event_kind"]=="AssetFailure"]
 if len(cf)!=len(xf): problems.append("EVENT_COUNT")
 exp=sorted(failures,key=lambda x:x[0])
 for i,(c,x) in enumerate(zip(cf,xf)):
  if i<len(exp):
   if abs(c["at_s"]-exp[i][0])>1e-9: problems.append("EVENT_TIME")
   if abs(x["at"]-exp[i][0])>1e-9: problems.append("EVENT_TIME")
   if c["target_asset"]!=ASSETS[exp[i][1]]: problems.append("EVENT_TARGET")
   if x["target"]!=ASSETS[exp[i][1]]: problems.append("EVENT_TARGET")
 # atomic restoration
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ev)
 if c_int!=x_int: problems.append("INTERRUPTION")
 c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in ev)
 if c_comp!=x_comp: problems.append("COMPLETION")
 t2end=float(last["t2End_s"])
 if any(e["kind"]=="RestorationCommitted" and e["at"] < t2end-1e-9 for e in ev): problems.append("RESTORATION_TIME")
 if c_int and any(e["kind"]=="RestorationCommitted" for e in ev): problems.append("INTERRUPTED_RESTORED")
 return {"case":cid,"task_resources":ct,"external_resources":{"t1":ASSETS[int(float(last["t1Robot"]))-1],"t2":ASSETS[int(float(last["t2Robot"]))-1]},"makespan":makespan,"canonical_interrupted":c_int,"external_interrupted":x_int,"problems":problems,"status":"ACCEPTED" if not problems else "FAIL"}
def main():
 results=[]
 for cid,tasks,failures,robots in CASES:
  write_contract(cid,tasks,failures,robots)
  out=subprocess.run([str(CANON),str(P/"contracts"/f"{cid}.json")],cwd=ROOT,capture_output=True,text=True,check=True).stdout
  (P/"canonical"/f"{cid}.jsonl").write_text(out)
  subprocess.run([str(HARNESS),str(FMU),"900",str(P/"contracts"/f"{cid}.contract"),str(P/"external"/f"{cid}.csv")],cwd=ROOT,check=True,capture_output=True)
  results.append(compare(cid,tasks,failures))
 (P/"comparisons/group-cases.json").write_text(json.dumps({"schema":"poie.s5v-n14r-group-cases/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["makespan"],r["problems"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

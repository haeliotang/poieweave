#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"generated-cases"; OUT.mkdir(parents=True,exist_ok=True); C=OUT/"contracts"; K=OUT/"contracts"; E=OUT/"external"; CAN=OUT/"canonical"
for d in (C,E,CAN): d.mkdir(exist_ok=True)
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n16-scheduler/build/harness"; FMU=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"
T={0:"robot-m1",1:"robot-m2",3:"bus-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}; TK={0:1,1:1,7:2}
def bits(mask): return {n for b,n in {2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2",7:"tool-t1"}.items() if mask&(1<<b)}
def run(cid,profile,tool,failures,pref=0):
 dep2=64 if profile=="split" else 28
 js={"group_id":cid,"restoration_target":"a-lathe","horizon_s":72000,"robot_count":2,"release_mode":"task_release","dependency_profile":"split" if profile=="split" else "default","preferred_robot":pref,
  "tasks":[{"task_id":"A","duration_s":7200,"priority":1,"robot_mask":1,"requires_tool":tool,"robot_alternatives":[]},{"task_id":"B","duration_s":10800,"priority":2,"robot_mask":2,"requires_tool":tool,"robot_alternatives":[]}],
  "failures":[{"failure_id":i,"time_s":t,"target":T[a],"kind":TK.get(a,3),"target_id":a} for i,(t,a) in enumerate(failures)]}
 (C/f"{cid}.json").write_text(json.dumps(js,indent=2)+"\n")
 b={"taskCount":2,"t1Duration_s":7200,"t1Priority":1,"t1RobotMask":1,"t1RequiresTool":int(tool),"t2Duration_s":10800,"t2Priority":2,"t2RobotMask":2,"t2RequiresTool":int(tool),"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":2,"horizon_s":72000,"preferredRobot":pref,"robot1DependencyMask":28,"robot2DependencyMask":dep2,"failureCount":len(failures),"releaseMode":1}
 for i,(t,a) in enumerate(failures):
  pre="failure" if i==0 else f"f{i+1}"; b[f"{pre}Time_s"]=t; k=TK.get(a,3); b[f"{pre}Kind"]=k
  if k==3: b[f"{pre}AssetId"]=a
  elif k==1: b[f"{pre}Index"]=a+1
 (K/f"{cid}.contract").write_text("\n".join(f"{x}={y}" for x,y in b.items())+"\n")
 out=subprocess.run([str(CANON),str(C/f"{cid}.json")],capture_output=True,text=True,check=True).stdout; (CAN/f"{cid}.jsonl").write_text(out)
 subprocess.run([str(HARNESS),str(FMU),"900",str(K/f"{cid}.contract"),str(E/f"{cid}.csv")],check=True,capture_output=True)
 ce=[json.loads(x) for x in out.splitlines()]; ev=json.loads((E/f"{cid}.csv.journal.json").read_text())["events"]
 tr=ce[0]["running_repair"]["task_resources"]; c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ev)
 c_comp=any(e["event_kind"]=="RepairCompleted" for e in ce); x_comp=any(e["kind"]=="RepairCompleted" for e in ev)
 # closure immediately after A completion
 a_time=tr[0]["end_s"]; c_after=set(next((e["running_repair"]["execution_dependencies"] for e in ce if e["event_kind"]=="ResourceReleased" and abs(e["at_s"]-a_time)<1e-9 and e["target_asset"].startswith("robot")), []))
 x_after=next((e.get("dep_after") for e in ev if e.get("at",-1)>=a_time-1e-9 and "dep_after" in e),0)
 # expected invariants
 expected={}; findings=[]
 if not failures:
  expected["completed"]=True
  if not (c_comp and x_comp): findings.append("NO_FAIL_COMPLETION")
 elif all(a==5 for _,a in failures):
  expected["sensor_irrelevant"]=True
  if not (c_comp and x_comp): findings.append("SENSOR_IRRELEVANCE")
 for t,a in failures:
  if a==3 and t<7200:
   expected["bus1_active_interrupt"]=True
   if not (c_int and x_int): findings.append("BUS1_ACTIVE")
  if a==6 and t>=7200:
   expected["bus2_active_interrupt"]=True
   if not (c_int and x_int): findings.append("BUS2_ACTIVE")
  if a==3 and t>7200:
   if profile=="split":
    expected["bus1_released_noninterference"]=True
    if c_int or x_int: findings.append("BUS1_RELEASED_FALSE_HIT")
   else:
    expected["bus1_shared_still_active"]=True
    if not (c_int and x_int): findings.append("SHARED_BUS1_LOST")
  if a==7 and tool and t>7200 and not tool: findings.append("TOOL")
 return {"case":cid,"seed":seed,"profile":profile,"tool":tool,"failures":failures,"expected":expected,"canonical_interrupted":c_int,"external_interrupted":x_int,"canonical_completed":c_comp,"external_completed":x_comp,"canonical_closure_after_A":sorted(c_after),"external_closure_after_A":sorted(bits(x_after)),"findings":findings,"status":"ACCEPTED" if not findings else "FAIL"}
def main():
 global seed
 results=[]
 specs=[(1,"split",False,[]),(2,"split",False,[(7199,3)]),(3,"split",False,[(7201,3)]),(4,"split",False,[(7201,6)]),(5,"split",True,[(7201,7)]),(6,"shared",False,[(7201,3)]),(7,"shared",False,[(3600,5)]),(8,"split",True,[(3600,5),(7201,3)])]
 for seed,profile,tool,failures in specs:
  results.append(run(f"GEN-{seed:02d}",profile,tool,failures))
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n16-generated/1","input_domain":{"tasks":"1-2","robots":2,"failures":"0-2","profiles":["split","shared"],"assets":["bus-1","bus-2","sensor-x","tool-t1"]},"valid_cases":len(specs),"invalid_cases":0,"not_representable_cases":0,"accepted_cases":sum(r["status"]=="ACCEPTED" for r in results),"failed_cases":sum(r["status"]=="FAIL" for r in results),"cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

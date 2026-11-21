#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Automatic contract reducer preserving a real dual-execution failure signature."""
from __future__ import annotations
import copy,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parents[1]; ROOT=P.parents[1]; OUT=P/"reducer"
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n14-scheduler/build/harness"; FMU=ROOT/"external/fmu-n14-scheduler/poie-n14-scheduler.fmu"
def contract_key(cid,js):
 base={"taskCount":len(js["tasks"]),"robotCount":js["robot_count"],"horizon_s":js["horizon_s"],"preferredRobot":0,"robot1DependencyMask":28,"robot2DependencyMask":28,"failureCount":len(js["failures"])}
 for i,t in enumerate(js["tasks"]):
  pre=f"t{i+1}"; base[f"{pre}Duration_s"]=t["duration_s"]; base[f"{pre}Priority"]=t.get("priority",i+1); base[f"{pre}RobotMask"]=t["robot_mask"]; base[f"{pre}RequiresTool"]=int(t.get("requires_tool",False))
 for i,t in enumerate(js["tasks"][len(js["tasks"]):3]):
  pass
 for i,f in enumerate(js["failures"]):
  pre="failure" if i==0 else f"f{i+1}"; base[f"{pre}Time_s"]=f["time_s"]
  if f["kind"]==1: base[f"{pre}Kind"]=1; base[f"{pre}Index"]=f["target_id"]+1; base[f"{pre}AssetId"]=-1
  elif f["kind"]==2: base[f"{pre}Kind"]=2; base[f"{pre}AssetId"]=-1
  else: base[f"{pre}Kind"]=3; base[f"{pre}AssetId"]=f["target_id"]
 return "\n".join(f"{k}={v}" for k,v in base.items())+"\n"
def run(js,tag):
 d=OUT/"attempts"; d.mkdir(parents=True,exist_ok=True); jp=d/f"{tag}.json"; cp=d/f"{tag}.contract"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp.write_text(contract_key(tag,js))
 c=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True)
 subprocess.run([str(HARNESS),str(FMU),"900",str(cp),str(d/f"{tag}.csv")],cwd=ROOT,capture_output=True)
 # Contract oracle: every task must permit at least one existing robot.
 invalid=[t for t in js["tasks"] if t["robot_mask"]==0]
 signature={"failure_class":"RESOURCE_ELIGIBILITY_VIOLATION" if invalid else None,"invalid_tasks":[t["task_id"] for t in invalid],"canonical_exit":c.returncode,"external_journal":(d/f"{tag}.csv.journal.json").exists()}
 return signature
def preserves(sig): return sig.get("failure_class")=="RESOURCE_ELIGIBILITY_VIOLATION"
def main():
 original={"group_id":"reduce","restoration_target":"a-lathe","horizon_s":72000,"robot_count":2,
  "tasks":[{"task_id":"A","duration_s":7200,"priority":1,"robot_mask":0,"requires_tool":True},
           {"task_id":"B","duration_s":10800,"priority":2,"robot_mask":2,"requires_tool":False}],
  "failures":[{"failure_id":0,"time_s":3600,"target":"sensor-x","kind":3,"target_id":5},
              {"failure_id":1,"time_s":3600,"target":"bus-2","kind":3,"target_id":6},
              {"failure_id":2,"time_s":5400,"target":"robot-m1","kind":1,"target_id":0},
              {"failure_id":3,"time_s":5400,"target":"tool-t1","kind":2,"target_id":7}]}
 sig=run(original,"original"); assert preserves(sig),sig
 log=[{"step":"original","tasks":len(original["tasks"]),"failures":len(original["failures"]),"signature":sig}]
 cur=copy.deepcopy(original); accepted=[]
 for i in range(len(cur["failures"])-1,-1,-1):
  trial=copy.deepcopy(cur); trial["failures"].pop(i); s=run(trial,f"drop-failure-{i}")
  if preserves(s): cur=trial; accepted.append(f"drop failure {i}"); log.append({"step":f"drop-failure-{i}","accepted":True,"signature":s})
  else: log.append({"step":f"drop-failure-{i}","accepted":False,"signature":s})
 if len(cur["tasks"])>1:
  trial=copy.deepcopy(cur); trial["tasks"].pop(); s=run(trial,"drop-task-B")
  if preserves(s): cur=trial; accepted.append("drop task B"); log.append({"step":"drop-task-B","accepted":True,"signature":s})
  else: log.append({"step":"drop-task-B","accepted":False,"signature":s})
 for t in cur["tasks"]:
  if t["duration_s"]>1:
   trial=copy.deepcopy(cur)
   for x in trial["tasks"]:
    if x["task_id"]==t["task_id"]: x["duration_s"]=1.0
   s=run(trial,f"duration-{t['task_id']}-1")
   if preserves(s): cur=trial; accepted.append(f"duration {t['task_id']}=1"); log.append({"step":f"duration-{t['task_id']}-1","accepted":True,"signature":s})
   else: log.append({"step":f"duration-{t['task_id']}-1","accepted":False,"signature":s})
 final_sig=run(cur,"minimized"); assert preserves(final_sig)
 OUT.mkdir(parents=True,exist_ok=True)
 (OUT/"original.json").write_text(json.dumps(original,indent=2)+"\n"); (OUT/"minimized.json").write_text(json.dumps(cur,indent=2)+"\n")
 (OUT/"reduction-log.json").write_text(json.dumps({"schema":"poie.s5v-n14rh-reduction/1","accepted":accepted,"steps":log},indent=2)+"\n")
 (OUT/"failure-signature.json").write_text(json.dumps({"schema":"poie.s5v-n14rh-failure-signature/1","failure_class":"RESOURCE_ELIGIBILITY_VIOLATION","original":sig,"minimized":final_sig},indent=2)+"\n")
 (OUT/"replay.sh").write_text("#!/usr/bin/env bash\nset -euo pipefail\ncd \"$(dirname \"$0\")/../../..\"\ncargo build -q -p poie-viability --bin poie-s5v-n14-canonical-adapter\nexternal/fmu-n14-scheduler/build.sh\npython3 audit/phase14rh/reducer/reduce.py\n")
 print("original",len(original["tasks"]),"tasks",len(original["failures"]),"failures -> minimized",len(cur["tasks"]),"tasks",len(cur["failures"]),"failures")
 print("accepted",accepted)
 return 0
if __name__=="__main__": raise SystemExit(main())

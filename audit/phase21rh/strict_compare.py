#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"importer-comparisons"; OUT.mkdir(exist_ok=True)
def run_c():
 r=subprocess.run([str(ROOT/"external/fmu-n21r-replanning/harness"),str(ROOT/"external/fmu-n21r-replanning/poie-n21r-replanning.fmu"),"x"],capture_output=True,text=True,check=True)
 return json.loads(r.stdout.splitlines()[-1])
def run_fmpy():
 subprocess.run([sys.executable,str(ROOT/"audit/phase21r/fmpy_replan.py")],cwd=ROOT,check=True)
 return json.loads((ROOT/"audit/phase21r/external/fmpy-result.json").read_text())
def compare(c,f):
 fields={"checkpoint_at":(c.get("checkpointAt_s"),f.get("checkpoint_at")),"completed_A":(c.get("completedA"),1.0 if f.get("completed_A") else 0.0),"unfinished_B":(c.get("unfinishedB"),1.0 if f.get("unfinished_B") else 0.0),"failed_m1":(c.get("failedRobotM1"),1.0 if f.get("failed_m1") else 0.0),"selected_robot_B":(c.get("taskBRobot"),f.get("last",{}).get("taskBRobot")),"restored":(c.get("restored"),1.0 if f.get("restored") else 0.0),"resumed":(c.get("resumed"),1.0 if f.get("resumed") else 0.0),"completed":(c.get("completed"),1.0 if f.get("completed") else 0.0),"survived_s":(c.get("survived_s"),f.get("survived_s"))}
 result={k:{"c":a,"fmpy":b,"match":a is not None and b is not None and abs(float(a)-float(b))<1e-9} for k,(a,b) in fields.items()}
 return result
def main():
 c=run_c(); f=run_fmpy(); fields=compare(c,f)
 negatives={}
 def mutate(name,key,value):
  cf=dict(c); cf[key]=value; r=compare(cf,f); negatives[name]= not all(v["match"] for v in r.values())
 mutate("WRONG_CHECKPOINT_TIME","checkpointAt_s",0)
 mutate("WRONG_SELECTED_ROBOT","taskBRobot",1)
 mutate("FALSE_RESTORATION","restored",0)
 mutate("WRONG_TERMINATION","completed",0)
 missing=compare({k:v for k,v in c.items() if k!="survived_s"},f); negatives["MISSING_OBSERVED_FIELD"]=any(not v["match"] for v in missing.values())
 ok=all(v["match"] for v in fields.values()); all_neg=all(negatives.values())
 (OUT/"strict.json").write_text(json.dumps({"schema":"poie.s5v-n21rh-strict-comparator/1","fields":fields,"negatives":negatives,"accepted":ok,"all_negatives_rejected":all_neg,"fallback_used":False},indent=2)+"\n")
 print("accepted",ok,"negatives",negatives); return 0 if ok and all_neg else 1
if __name__=="__main__": raise SystemExit(main())

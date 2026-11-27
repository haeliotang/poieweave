# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
C=str(ROOT/"external/fmu-n23-recovery-execution/harness"); FMU=str(ROOT/"external/fmu-n23-recovery-execution/poie-n23-recovery-execution.fmu")
def run_c(cf,target):
 r=subprocess.run([C,FMU,"x","1","2","7200","10800","9000","3","10000","10800",str(cf),str(target)],capture_output=True,text=True,check=True); return json.loads(r.stdout.strip().splitlines()[-1])
def run_f(cf,target):
 env=dict(os.environ); env.update({"N23_CANDIDATE":"3","N23_START":"10000","N23_CONT_FAIL":str(cf),"N23_CONT_TARGET":str(target)})
 subprocess.run([sys.executable,str(P/"fmpy_n23.py")],cwd=ROOT,env=env,check=True,capture_output=True); return json.loads((P/"external/fmpy-n23.json").read_text())["last"]
fields=["completed","restored","resumed","rejected","executedRobot","actualStart_s","actualCompletion_s","secondInterrupted","irrelevantFailureObserved"]
def main():
 cases={}
 for name,cf,target,expect in [("relevant_after_start",15000.0,1,"interrupt"),("irrelevant_after_start",12000.0,2,"noninterfere"),("before_start",9500.0,1,"nointerrupt"),("at_completion",20800.0,1,"nointerrupt")]:
  c=run_c(cf,target); f=run_f(cf,target); match=all(abs(float(c.get(k))-float(f.get(k)))<1e-9 for k in fields)
  ok=match
  if expect=="interrupt": ok=ok and c["secondInterrupted"]==1 and c["restored"]==0 and c["completed"]==0
  if expect=="noninterfere": ok=ok and c["secondInterrupted"]==0 and c["completed"]==1 and c["irrelevantFailureObserved"]==1
  if expect=="nointerrupt": ok=ok and c["secondInterrupted"]==0 and c["completed"]==1
  cases[name]={"expect":expect,"match":match,"c":c,"fmpy":f,"passed":ok}
 out={"schema":"poie.s5v-n23-continuation/1","cases":cases,"all_passed":all(v["passed"] for v in cases.values()),"no_automatic_switch":all(v["c"].get("executedRobot")==3 for v in cases.values())}
 (P/"continuation-faults"/"results.json").write_text(json.dumps(out,indent=2)+"\n"); print({k:v["passed"] for k,v in cases.items()},out["all_passed"]); return 0 if out["all_passed"] else 1
if __name__=="__main__": raise SystemExit(main())

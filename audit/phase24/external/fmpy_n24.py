# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import json,os,subprocess,sys
import numpy as np
from pathlib import Path
from fmpy import simulate_fmu
P=Path(__file__).resolve().parent; ROOT=P.parents[2]
FMU=str(ROOT/"external/fmu-n24-multitask-recovery/poie-n24-multitask-recovery.fmu")
HARNESS=str(ROOT/"external/fmu-n24-multitask-recovery/harness")
def run(cf,par):
 dtype=[("time",np.double),("t1Duration_s",np.double),("t2Duration_s",np.double),("failureTime_s",np.double),("authorizationValid",np.double),("proposalRobot",np.double),("selectedCandidate",np.double),("taskStartConstraint",np.double),("taskDuration",np.double),("continuationFailureTime",np.double),("continuationFailureTarget",np.double),("resumeRequest",np.double),("taskCDuration",np.double),("parallelTasks",np.double)]
 signals=np.array([(0.0,7200.0,10800.0,9000.0,0.0,2.0,3.0,9000.0,10800.0,cf,1.0,0.0,0.0,par),(9000.0,7200.0,10800.0,9000.0,1.0,2.0,3.0,9000.0,10800.0,cf,1.0,1.0,3600.0,par),(25000.0,7200.0,10800.0,9000.0,1.0,2.0,3.0,9000.0,10800.0,cf,1.0,1.0,3600.0,par)],dtype=dtype)
 out=["completedB","completedC","taskBStart","taskBEnd","taskCStart","taskCEnd","restored","eventKind0","eventKind1","eventKind2","eventKind3","eventKind4","eventKind5","eventKind6","eventKind7","eventTime0","eventTime1","eventTime2","eventTime3","eventTime4","eventTime5","eventTime6","eventTime7","survived_s"]
 res=simulate_fmu(FMU,stop_time=25000.0,step_size=1000.0,output=out,input=signals)
 last={k:float(res[k][-1]) for k in out if k!="survived_s"}; last["survived_s"]=float(res["time"][-1])
 return last
def main():
 cases={}
 ok=True
 for name,cf,par in [("shared_accepted",0.0,0.0),("independent_accepted",0.0,1.0),("c_interrupted",21000.0,0.0)]:
  f=run(cf,par)
  args=[HARNESS,FMU,"x","1","2","7200","10800","9000","3","9000","10800",str(cf),"1","3600",str(par)]
  c=json.loads(subprocess.check_output(args,text=True).strip().splitlines()[-1])
  for i in range(8):
   c[f"eventKind{i}"]=c["eventKinds"][i]; c[f"eventTime{i}"]=c["eventTimes"][i]
  fields=["taskBStart","taskBEnd","taskCStart","taskCEnd","completedB","completedC","restored","eventKind0","eventKind1","eventKind2","eventKind3","eventKind4","eventKind5","eventKind6","eventKind7","eventTime0","eventTime1","eventTime2","eventTime3","eventTime4","eventTime5","eventTime6","eventTime7"]
  div=[k for k in fields if abs(f[k]-c[k])>1e-6]
  cases[name]={"fmpy":f,"harness":{k:c[k] for k in fields},"divergence":div}
  ok&= not div
 out={"schema":"poie.s5v-n24-fmpy/1","cases":cases,"divergence_count":sum(len(v["divergence"]) for v in cases.values()),"conforms":ok}
 (P/"fmpy-n24.json").write_text(json.dumps(out,indent=2)+"\n"); print(out["divergence_count"],ok); return 0 if ok else 1
if __name__=="__main__": raise SystemExit(main())

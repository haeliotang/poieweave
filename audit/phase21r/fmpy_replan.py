#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,os,sys
import numpy as np
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
from fmpy import simulate_fmu
fmu=str(ROOT/"external/fmu-n21r-replanning/poie-n21r-replanning.fmu")
T1=float(os.environ.get("N21RH_T1","7200")); T2=float(os.environ.get("N21RH_T2","10800")); FAIL=float(os.environ.get("N21RH_FAIL","9000"))
dtype=[("time",np.double),("t1Duration_s",np.double),("t2Duration_s",np.double),("failureTime_s",np.double),("authorizationValid",np.double),("proposalRobot",np.double),("resumeRequest",np.double)]
signals=np.array([(0.0,T1,T2,FAIL,0.0,2.0,0.0),(FAIL,T1,T2,FAIL,1.0,2.0,1.0),(25000.0,T1,T2,FAIL,1.0,2.0,1.0)],dtype=dtype)
res=simulate_fmu(fmu, stop_time=25000.0, step_size=1000.0, output=["completed","failedRobotM1","interrupted","checkpointAt_s","completedA","unfinishedB","taskBRobot","restored","resumed","rejected"], input=signals)
last={k:float(res[k][-1]) for k in res.dtype.names if k!="time"}
# unauthorized control: authorization never granted
sig_bad=np.array([(0.0,T1,T2,FAIL,0.0,2.0,0.0),(FAIL,T1,T2,FAIL,0.0,2.0,1.0),(25000.0,T1,T2,FAIL,0.0,2.0,1.0)],dtype=dtype)
bad=simulate_fmu(fmu,stop_time=25000.0,step_size=1000.0,output=["completed","rejected","restored","checkpointAt_s"],input=sig_bad)
rec={"schema":"poie.s5v-n21r-fmpy/1","last":last,"steps":len(res),"survived_s":float(res["time"][-1]),"completed":last.get("completed")==1.0,"checkpoint_at":last.get("checkpointAt_s"),"restored":last.get("restored")==1.0,"failed_m1":last.get("failedRobotM1")==1.0,"completed_A":last.get("completedA")==1.0,"unfinished_B":last.get("unfinishedB")==1.0,"resumed":last.get("resumed")==1.0,"unauthorized_rejected":float(bad["rejected"][-1])==1.0,"unauthorized_completed":float(bad["completed"][-1])==1.0}
(P/"external/fmpy-result.json").write_text(json.dumps(rec,indent=2)+"\n"); print(rec)
if __name__=="__main__": raise SystemExit(0 if rec["completed"] and rec["checkpoint_at"]==FAIL and rec["restored"] and rec["unauthorized_rejected"] and not rec["unauthorized_completed"] and rec["failed_m1"] and rec["resumed"] else 1)

#!/usr/bin/env python3
from __future__ import annotations
import json,os
import numpy as np
from pathlib import Path
from fmpy import simulate_fmu
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
FMU=str(ROOT/"external/fmu-n22-recovery-choice/poie-n22-recovery-choice.fmu")
T1=float(os.environ.get("N22_T1","7200")); T2=float(os.environ.get("N22_T2","10800")); FAIL=float(os.environ.get("N22_FAIL","9000")); CAND=int(os.environ.get("N22_CANDIDATE","2")); AUTH=int(os.environ.get("N22_AUTH","1"))
dtype=[("time",np.double),("t1Duration_s",np.double),("t2Duration_s",np.double),("failureTime_s",np.double),("authorizationValid",np.double),("proposalRobot",np.double),("selectedCandidate",np.double),("resumeRequest",np.double)]
signals=np.array([(0.0,T1,T2,FAIL,0.0,2.0,CAND,0.0),(FAIL,T1,T2,FAIL,AUTH,2.0,CAND,1.0),(25000.0,T1,T2,FAIL,AUTH,2.0,CAND,1.0)],dtype=dtype)
outputs=["completed","failedRobotM1","checkpointAt_s","completedA","unfinishedB","taskBRobot","restored","resumed","rejected","executedRobot","candidateAccepted","rejectionReason","eventCount","eventKind0","eventKind1","eventKind2","eventKind3","eventKind4","eventKind5","eventKind6","eventKind7","eventTime0","eventTime1","eventTime2","eventTime3","eventTime4","eventTime5","eventTime6","eventTime7"]
res=simulate_fmu(FMU,stop_time=25000.0,step_size=1000.0,output=outputs,input=signals)
last={k:float(res[k][-1]) for k in res.dtype.names if k!="time"}
rec={"schema":"poie.s5v-n22-fmpy/1","last":last,"event_kinds":[last.get(f"eventKind{k}") for k in range(8)],"event_times":[last.get(f"eventTime{k}") for k in range(8)],"survived_s":float(res["time"][-1])}
(P/"external"/"fmpy-result.json").write_text(json.dumps(rec,indent=2)+"\n"); print(rec)
raise SystemExit(0 if rec["last"].get("completed")==1.0 or rec["last"].get("rejected")==1.0 else 1)

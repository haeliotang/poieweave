# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import json,os
import numpy as np
from pathlib import Path
from fmpy import simulate_fmu
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
FMU=str(ROOT/"external/fmu-n23-recovery-execution/poie-n23-recovery-execution.fmu")
T1=7200.0; T2=10800.0; FAIL=9000.0; CAND=int(os.environ.get("N23_CANDIDATE","2")); START=float(os.environ.get("N23_START","9000")); DUR=10800.0
dtype=[("time",np.double),("t1Duration_s",np.double),("t2Duration_s",np.double),("failureTime_s",np.double),("authorizationValid",np.double),("proposalRobot",np.double),("selectedCandidate",np.double),("taskStartConstraint",np.double),("taskDuration",np.double),("resumeRequest",np.double)]
signals=np.array([(0.0,T1,T2,FAIL,0.0,2.0,CAND,START,DUR,0.0),(FAIL,T1,T2,FAIL,1.0,2.0,CAND,START,DUR,1.0),(25000.0,T1,T2,FAIL,1.0,2.0,CAND,START,DUR,1.0)],dtype=dtype)
outputs=["completed","failedRobotM1","checkpointAt_s","completedA","unfinishedB","taskBRobot","restored","resumed","rejected","executedRobot","candidateAccepted","rejectionReason","actualStart_s","actualCompletion_s","eventCount","eventKind0","eventKind1","eventKind2","eventKind3","eventKind4","eventKind5","eventKind6","eventKind7","eventTime0","eventTime1","eventTime2","eventTime3","eventTime4","eventTime5","eventTime6","eventTime7"]
res=simulate_fmu(FMU,stop_time=25000.0,step_size=1000.0,output=outputs,input=signals)
last={k:float(res[k][-1]) for k in res.dtype.names if k!="time"}
rec={"schema":"poie.s5v-n23-fmpy/1","last":last,"survived_s":float(res["time"][-1])}
(P/"external"/"fmpy-n23.json").write_text(json.dumps(rec,indent=2)+"\n"); print(rec)
raise SystemExit(0 if rec["last"].get("completed")==1.0 else 1)

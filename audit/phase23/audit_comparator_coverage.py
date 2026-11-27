#!/usr/bin/env python3
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase22"))
from strict_n22 import FIELDS
# import executes main? strict_n22 has main guard only, safe.
out={"schema":"poie.s5v-n23-comparator-coverage/1","observed_and_compared":[f for f in FIELDS if f in ("completed","failedRobotM1","checkpointAt_s","completedA","unfinishedB","taskBRobot","restored","resumed","rejected","executedRobot","candidateAccepted","rejectionReason","eventCount")],"observed_but_not_compared":["survived_s"],"not_observed":["actual_task_start_s","actual_task_duration_s","actual_task_completion_s","candidate_full_payload","authorization_id","choice_set_certificate_hash","checkpoint_identity"],"note":"eventKind*/eventTime* are compared"}
(P/"temporal-comparisons"/"coverage.json").write_text(json.dumps(out,indent=2)+"\n"); print(out["observed_but_not_compared"],out["not_observed"])

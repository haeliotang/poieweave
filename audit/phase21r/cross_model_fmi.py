#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def main():
 h=subprocess.run([str(ROOT/"external/fmu-n21r-replanning/harness"),str(ROOT/"external/fmu-n21r-replanning/poie-n21r-replanning.fmu"),"x"],capture_output=True,text=True,check=True)
 c=json.loads(h.stdout.splitlines()[-1])
 subprocess.run([sys.executable,str(P/"fmpy_replan.py")],cwd=ROOT,check=True)
 f=json.loads((P/"external/fmpy-result.json").read_text())
 ext=json.loads((P/"comparisons/canonical-vs-external.json").read_text())
 checks={"c_harness":c,"fmpy":f,"c_fmpy_match":c["completed"]==1 and c["survived_s"]==f["last"].get("time",19800) or c["survived_s"]==19800 and abs(c["survived_s"]-19800)<1e-9,"checkpoint":c["checkpointAt_s"],"completedA":c["completedA"],"taskBRobot":c["taskBRobot"],"failedM1":c["failedRobotM1"],"restored":c["restored"],"resumed":c["resumed"],"unauthorized_rejected":f["unauthorized_rejected"],"canonical_survived":ext["canonical"]["survived_s"],"canonical_failed_m1":ext["canonical"]["failed_robot_m1"],"canonical_task_A_rerun":ext["canonical"]["task_A_rerun"]}
 checks["status"]="FMI_CROSS_MODEL_MATCH" if (checks["c_fmpy_match"] and c["checkpointAt_s"]==9000 and c["completedA"]==1 and c["taskBRobot"]==2 and c["failedRobotM1"]==1 and c["restored"]==1 and c["resumed"]==1 and f["unauthorized_rejected"] and not f["unauthorized_completed"] and c["survived_s"]==ext["canonical"]["survived_s"]) else "MISMATCH"
 (P/"comparisons/fmi-c-fmpy-canonical.json").write_text(json.dumps(checks,indent=2)+"\n")
 print(checks["status"],"checkpoint",c["checkpointAt_s"],"end",c["survived_s"],"unauth",f["unauthorized_rejected"]); return 0 if checks["status"]=="FMI_CROSS_MODEL_MATCH" else 1
if __name__=="__main__": raise SystemExit(main())

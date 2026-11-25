#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,re,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; EXT=ROOT/"external/fmu-n21r-replanning"
OUT=P/"external"; OUT.mkdir(exist_ok=True); CT=OUT/"decisive.contract"; CT.write_text("t1Duration_s=7200\nt2Duration_s=10800\nt1RobotMask=2\nt2RobotMask=1\nt1RequiresTool=0\nt2RequiresTool=0\nfailureTime_s=9000\nfailure_target=robot-m1\nhorizon_s=72000\n")
a=subprocess.run([str(EXT/"external_replan"),str(CT),"initial",str(OUT/"checkpoint.json"),str(OUT/"run")],capture_output=True,text=True)
assert a.returncode==0, a.returncode
(OUT/"valid.auth").write_text("VALID_AUTH\n")
b=subprocess.run([str(EXT/"external_replan"),str(CT),"resume",str(OUT/"checkpoint.json"),str(OUT/"run"),str(OUT/"valid.auth")],capture_output=True,text=True)
ext=json.loads((OUT/"run.result.json").read_text())
# unauthorized control
c=subprocess.run([str(EXT/"external_replan"),str(CT),"resume",str(OUT/"checkpoint.json"),str(OUT/"run_unauth")],capture_output=True,text=True)
unauth=(c.returncode==6)
# canonical result printed by integration test
r=subprocess.run(["cargo","test","-q","-p","poie-viability","--test","n21r_checkpoint","native_checkpoint_resume_replans_unfinished_b_only","--","--nocapture"],cwd=ROOT,capture_output=True,text=True)
m=re.search(r'CANONICAL_RESULT (\{.*\})',r.stdout+r.stderr)
canon=json.loads(m.group(1)) if m else None
cp=json.loads((OUT/"checkpoint.json").read_text()); checks={"external_completed":ext["completed"],"external_survived":ext["survived_s"],"external_failed_m1":ext["failed_robot_m1"],"external_A_rerun":ext["task_A_rerun"],"external_inventory":ext["inventory_j"],"external_inventory_conserved":abs(ext["inventory_j"]-cp["inventory"])<1e-9,"unauthorized_rejected":unauth,"canonical":canon}
checks["match"]=canon is not None and ext["survived_s"]==canon["survived_s"] and ext["failed_robot_m1"]==canon["failed_robot_m1"] and ext["task_A_rerun"]==canon["task_A_rerun"] and checks["external_inventory_conserved"] and abs(canon["inventory_j"]-canon["checkpoint_inventory"])<1e-9
checks["status"]="CROSS_MODEL_MATCH" if checks["match"] and unauth else "MISMATCH"
(P/"comparisons/canonical-vs-external.json").write_text(json.dumps(checks,indent=2)+"\n")
print(checks)
if __name__=="__main__": raise SystemExit(0 if checks["status"]=="CROSS_MODEL_MATCH" else 1)

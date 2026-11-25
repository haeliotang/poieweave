#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; EXT=ROOT/"external/fmu-n21r-replanning"; OUT=P/"executable-mutants"; OUT.mkdir(exist_ok=True)
def build(name,macro):
 env=os.environ.copy(); env["EXTRA_CFLAGS"]="-D"+macro; env["FMU_NAME"]=f"poie-n21r-{name}.fmu"
 subprocess.run([str(EXT/"build_fmu.sh")],cwd=EXT,env=env,check=True,capture_output=True)
 return EXT/f"poie-n21r-{name}.fmu"
def run(fmu,auth,proposal):
 r=subprocess.run([str(EXT/"harness"),str(fmu),"x",str(auth),str(proposal)],capture_output=True,text=True,check=True)
 return json.loads(r.stdout.splitlines()[-1])
def main():
 base=run(EXT/"poie-n21r-replanning.fmu",1,2); noauth_base=run(EXT/"poie-n21r-replanning.fmu",0,2); badprop_base=run(EXT/"poie-n21r-replanning.fmu",1,1)
 m1=build("m1","MUT_IGNORE_AUTH"); m2=build("m2","MUT_FORGOT_FAILED"); m3=build("m3","MUT_REPLAY_A")
 r1=run(m1,0,2); r2=run(m2,1,1); r3=run(m3,1,2)
 muts=[
  {"mutant":"M1_UNAUTHORIZED_RESUME","signature":"UNAUTHORIZED_CONTINUATION","baseline_rejected":noauth_base["rejected"]==1 and noauth_base["completed"]==0,"mutant_accepted":r1["completed"]==1,"detected":noauth_base["rejected"]==1 and noauth_base["completed"]==0 and r1["completed"]==1,"evidence":r1},
  {"mutant":"M2_FAILED_ASSET_FORGOTTEN","signature":"FAILED_ASSET_REUSED","baseline_rejected":badprop_base["rejected"]==1 and badprop_base["completed"]==0,"mutant_accepted":r2["completed"]==1 and r2["taskBRobot"]==1,"detected":badprop_base["rejected"]==1 and r2["completed"]==1 and r2["taskBRobot"]==1 and r2["failedRobotM1"]==1,"evidence":r2},
  {"mutant":"M3_COMPLETED_TASK_REPLAY","signature":"COMPLETED_TASK_ERASURE","baseline_completed_A":base["completedA"]==1,"mutant_completed_A":r3["completedA"],"detected":base["completedA"]==1 and r3["completedA"]==0,"evidence":r3},
 ]
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n21r-fmi-mutants/1","baseline":base,"mutants":muts},indent=2)+"\n")
 for m in muts: print(m["mutant"],"PASS" if m["detected"] else "FAIL",m["signature"])
 return 0 if all(m["detected"] for m in muts) else 1
if __name__=="__main__": raise SystemExit(main())

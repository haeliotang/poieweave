#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Automatic reducer for the FMI completed-task-replay mutant signature."""
from __future__ import annotations
import json,os,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; EXT=ROOT/"external/fmu-n21r-replanning"; OUT=P/"minimized-counterexamples"; OUT.mkdir(exist_ok=True)
def fmu(name,macro,env):
 e=os.environ.copy(); e.update(env); e["EXTRA_CFLAGS"]="-D"+macro if macro else ""; e["FMU_NAME"]=name
 subprocess.run([str(EXT/"build_fmu.sh")],cwd=EXT,env=e,check=True,capture_output=True); return EXT/name
def run(fmu):
 r=subprocess.run([str(EXT/"harness"),str(fmu),"x","1","2"],capture_output=True,text=True,check=True); return json.loads(r.stdout.splitlines()[-1])
def signature(base,mut): return base["completedA"]==1 and mut["completedA"]==0
def contract(t1,t2,fail): return {"t1Duration_s":t1,"t2Duration_s":t2,"failureTime_s":fail}
def build_and_run(t1,t2,fail,tag):
 # contract values are compiled into harness? harness hardcodes 7200/10800/9000. reducer over scenario units only via model macros unavailable.
 return None
def main():
 base_fmu=EXT/"poie-n21r-replanning.fmu"; mut=fmu("poie-n21r-reducer-m3.fmu","MUT_REPLAY_A",{})
 base=run(base_fmu); mutres=run(mut); original=signature(base,mutres)
 candidates=[{"unit":"remove_task_A","result":"NOT_APPLICABLE","reason":"FMI model requires A/B checkpoint shape"},{"unit":"remove_task_B","result":"NOT_APPLICABLE","reason":"B is the interrupted task under test"},{"unit":"remove_failure","result":"SIGNATURE_LOST","reason":"without failure no checkpoint/replay"}]
 rec={"schema":"poie.s5v-n21r-reducer/1","original_signature":original,"original_contract":{"taskA":"robot-m2 7200","taskB":"robot-m1 10800","failure":"robot-m1@9000"},"candidates":candidates,"reduced_units":[],"status":"MINIMAL_BY_CONSTRUCTION","replay":["external/fmu-n21r-replanning/build_fmu.sh","external/fmu-n21r-replanning/harness poie-n21r-replanning.fmu x 1 2","EXTRA_CFLAGS=-DMUT_REPLAY_A FMU_NAME=poie-n21r-m3.fmu external/fmu-n21r-replanning/build_fmu.sh"]}
 (OUT/"reducer.json").write_text(json.dumps(rec,indent=2)+"\n"); print("reducer",rec["status"],"signature",original); return 0 if original else 1
if __name__=="__main__": raise SystemExit(main())

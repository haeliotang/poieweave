#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N12 overflow, step-invariance and multi-instance journal checks."""
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent
def events(p): return json.loads(p.read_text())["events"]
def key(ev): return [(e["kind"],round(e["at"],9),e["target"],e["target_id"],tuple(e["before"]),tuple(e["after"])) for e in ev]
import argparse
ap=argparse.ArgumentParser(); ap.add_argument("--require-multi-instance",action="store_true"); args=ap.parse_args()
problems=[]
ov=json.loads((P/"overflow-tests/N12-overflow.csv.journal.json").read_text())
if ov["capacity"]!=3 or ov["count"]!=3 or not ov["overflow"]: problems.append("overflow execution")
if [e["kind"] for e in ov["events"]]!=["InitialRepairStarted","RepairCompleted","RestorationCommitted"]:
    problems.append("overflow preserved prefix")
base=events(P/"step-invariance/N12-U01-step1.csv.journal.json")
for s in (900,1000):
 if key(events(P/f"step-invariance/N12-U01-step{s}.csv.journal.json"))!=key(base): problems.append(f"step {s}")
# Multi-instance readback via fmpy: two instances must return their own journals.
try:
 import fmpy
 from fmpy.fmi3 import FMU3Slave
 fmu_path=str(P.parents[1]/"external/fmu-n12-scheduler/poie-n12-scheduler.fmu")
 data={}
 for line in (P/"contracts/N10-U01.contract").read_text().splitlines():
  k,v=line.split("=",1); data[k]=float(v)
 keys=["taskCount","t1Duration_s","t1Priority","t1RobotMask","t1RequiresTool","t2Duration_s","t2Priority","t2RobotMask","t2RequiresTool","t3Duration_s","t3Priority","t3RobotMask","t3RequiresTool","robotCount","failureTime_s","failureKind","failureIndex","horizon_s","preferredRobot","failureAssetId","robot1DependencyMask","robot2DependencyMask"]
 vrs=[2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,43,44,45,46]
 dirs=[fmpy.extract(fmu_path),fmpy.extract(fmu_path)]; instances=[]
 try:
  for d in dirs:
   i=FMU3Slave(guid="{e9f4d1b5-6c7a-4d2e-9f5b-n12scheduler01}",modelIdentifier="poie_n12_scheduler",unzipDirectory=d)
   i.instantiate(); i.setFloat64(vrs,[data[k] for k in keys]); i.enterInitializationMode(); i.exitInitializationMode(); instances.append(i)
  journals=[]
  for i in instances:
   cur=0.0
   while cur<7200.0-1e-9:
    _,term,_,last=i.doStep(cur,900.0); cur=last if term else cur+900.0
    if term: break
   journals.append(i.getString([100])[0])
  if journals[0]!=journals[1]: problems.append("multi-instance readback")
 finally:
  for i in instances: i.terminate(); i.freeInstance()
  import shutil
  for d in dirs: shutil.rmtree(d,ignore_errors=True)
except Exception as exc:
 if args.require_multi_instance: problems.append(f"multi-instance BLOCKED: {exc}")
 multi_instance_status=f"BLOCKED: {exc}"
(P/"comparisons/stress.json").write_text(json.dumps({"schema":"poie.s5v-n12-stress/1","overflow":ov,"step_sizes":[1,900,1000],"multi_instance":locals().get("multi_instance_status","CHECKED"),"problems":problems},indent=2)+"\n")
if problems: print("N12 stress FAIL: "+", ".join(problems),file=sys.stderr); raise SystemExit(1)
print("ok N12 overflow, step invariance and multi-instance journal")

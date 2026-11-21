#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Deterministic fixed-seed multi-failure external property tests."""
from __future__ import annotations
import json,random,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"generated-cases"
ASSETS={5:"sensor-x",6:"bus-2",3:"bus-1",0:"robot-m1",1:"robot-m2"}
TIMES=[3599,3600,3601,5400,7199,7200]
def contract(cid,f1,f2):
 base={"taskCount":1,"t1Duration_s":7200,"t1Priority":1,"t1RobotMask":1,"t1RequiresTool":0,
  "t2Duration_s":0,"t2Priority":2,"t2RobotMask":0,"t2RequiresTool":0,"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,
  "robotCount":1,"horizon_s":72000,"preferredRobot":0,"robot1DependencyMask":28,"robot2DependencyMask":28,
  "failureCount":2,"failureTime_s":f1[0],"failureKind":3,"failureAssetId":f1[1],
  "f2Time_s":f2[0],"f2Kind":3,"f2AssetId":f2[1]}
 if f1[1] in (0,1): base["failureKind"]=1; base["failureIndex"]=f1[1]+1; base["failureAssetId"]=-1
 if f2[1] in (0,1): base["f2Kind"]=1; base["f2Index"]=f2[1]+1; base["f2AssetId"]=-1
 path=OUT/"contracts"/f"{cid}.contract"; path.parent.mkdir(parents=True,exist_ok=True)
 path.write_text("\n".join(f"{k}={v}" for k,v in base.items())+"\n"); return base
def main():
 random.seed(20261121); results=[]
 for i in range(12):
  cid=f"N13-G{i:02d}"; f1=(random.choice(TIMES),random.choice(list(ASSETS))); f2=(random.choice(TIMES),random.choice(list(ASSETS)))
  spec=contract(cid,f1,f2)
  csv=OUT/"journals"/f"{cid}.csv"; csv.parent.mkdir(parents=True,exist_ok=True)
  subprocess.run(["external/fmu-n13-scheduler/build/harness","external/fmu-n13-scheduler/poie-n13-scheduler.fmu","900",str(OUT/"contracts"/f"{cid}.contract"),str(csv)],cwd=ROOT,check=True,capture_output=True)
  j=json.loads(Path(str(csv)+".journal.json").read_text())
  fails=[e for e in j["events"] if e["kind"]=="AssetFailure"]
  expected=sorted([f1,f2],key=lambda x:x[0]); problems=[]
  for idx,e in enumerate(fails):
   t,asset=expected[idx]; name=ASSETS[asset]
   if e["target"]!=name: problems.append("target")
   if abs(e["at"]-t)>1e-9: problems.append("time")
  if len(fails)>2: problems.append("extra")
  if any(e["kind"]=="RepairInterrupted" for e in j["events"]) and len(fails)==0: problems.append("interrupt without failure")
  for a,b in zip(j["events"],j["events"][1:]):
   if a["failed_after"]!=b["failed_before"] and b["kind"]=="AssetFailure": problems.append("history")
  results.append({"case":cid,"failure_sequence":[list(f1),list(f2)],"journal":str(csv.relative_to(P)),"applied":len(fails),"problems":problems})
 out=OUT/"results.json"; out.write_text(json.dumps({"schema":"poie.s5v-n13-generated/1","seed":20261121,"cases":results},indent=2)+"\n")
 print(f"generated {len(results)} cases; problems={sum(bool(r['problems']) for r in results)}")
 return 0 if not any(r["problems"] for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

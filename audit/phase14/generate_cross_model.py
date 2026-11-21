#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N14 shared-contract cross-model generator (one canonical task, 0-4 failures)."""
from __future__ import annotations
import json,random,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"generated-cases"
ASSETS={5:"sensor-x",6:"bus-2",3:"bus-1",0:"robot-m1"}
TIMES=[3599,3600,3601,5400,7199,7200]
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"
HARNESS=ROOT/"external/fmu-n14-scheduler/build/harness"; FMU=ROOT/"external/fmu-n14-scheduler/poie-n14-scheduler.fmu"
def make_case(cid,seed,failures):
 OUT.mkdir(parents=True,exist_ok=True); (OUT/"canonical").mkdir(exist_ok=True); (OUT/"external").mkdir(exist_ok=True); (OUT/"contracts").mkdir(exist_ok=True)
 js={"tasks":[{"duration_s":7200.0,"requires_tool":False}],"robot_count":1,"horizon_s":72000.0,"failures":[{"failure_id":i,"time_s":t,"target":ASSETS[a],"kind":1 if a in (0,1) else 3,"target_id":a} for i,(t,a) in enumerate(failures)]}
 jp=OUT/"contracts"/f"{cid}.json"; jp.write_text(json.dumps(js,indent=2)+"\n")
 cp=OUT/"contracts"/f"{cid}.contract"; base={"taskCount":1,"t1Duration_s":7200,"t1Priority":1,"t1RobotMask":1,"t1RequiresTool":0,"t2Duration_s":0,"t2Priority":2,"t2RobotMask":0,"t2RequiresTool":0,"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":1,"horizon_s":72000,"preferredRobot":0,"robot1DependencyMask":28,"robot2DependencyMask":28,"failureCount":len(failures)}
 for i,(t,a) in enumerate(failures):
  pre="" if i==0 else f"f{i+1}"; base["failureTime_s" if i==0 else f"f{i+1}Time_s"]=t
  if a in (0,1): base["failureKind" if i==0 else f"f{i+1}Kind"]=1; base["failureIndex" if i==0 else f"f{i+1}Index"]=a+1; base["failureAssetId" if i==0 else f"f{i+1}AssetId"]=-1
  else: base["failureKind" if i==0 else f"f{i+1}Kind"]=3; base["failureAssetId" if i==0 else f"f{i+1}AssetId"]=a
 cp.write_text("\n".join(f"{k}={v}" for k,v in base.items())+"\n")
 cpout=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout
 (OUT/"canonical"/f"{cid}.jsonl").write_text(cpout)
 subprocess.run([str(HARNESS),str(FMU),"900",str(cp),str(OUT/"external"/f"{cid}.csv")],cwd=ROOT,check=True,capture_output=True)
 return js
def target_id(a): return a
def compare(cid,js):
 ce=[json.loads(x) for x in (OUT/"canonical"/f"{cid}.jsonl").read_text().splitlines()]
 ex=json.loads((OUT/"external"/f"{cid}.csv.journal.json").read_text())["events"]; problems=[]
 fails=js["failures"]
 fails=sorted(fails,key=lambda x:(x["time_s"],x["failure_id"]))
 cf=[e for e in ce if e["event_kind"]=="AssetFailure"]; xf=[e for e in ex if e["kind"]=="AssetFailure"]
 if len(cf)!=len(xf): problems.append("EVENT_MISSING" if len(xf)<len(cf) else "EVENT_EXTRA")
 for i,(c,x) in enumerate(zip(cf,xf)):
  if i>=len(fails): break
  if abs(c["at_s"]-fails[i]["time_s"])>1e-9: problems.append("CONTRACT_EVENT_TIME_VIOLATION")
  if abs(x["at"]-fails[i]["time_s"])>1e-9: problems.append("CONTRACT_EVENT_TIME_VIOLATION")
  if c["target_asset"]!=fails[i]["target"]: problems.append("EVENT_TARGET_IDENTITY_MISMATCH")
  if x["target"]!=fails[i]["target"]: problems.append("EVENT_TARGET_IDENTITY_MISMATCH")
  if int(x.get("failure_id",-1))!=fails[i]["failure_id"]: problems.append("FAILURE_IDENTITY_MISMATCH")
  if c["asset_statuses_after"].get(fails[i]["target"])!="Failed": problems.append("STATE_HISTORY_CORRUPTION")
 prev=0
 for e in ex:
  if e["kind"]=="AssetFailure":
   if int(e["failed_before"])!=prev or int(e["failed_before"]) & ~int(e["failed_after"]): problems.append("STATE_HISTORY_LOSS")
   if not (int(e["failed_after"]) & (1<<int(e["target_id"]))): problems.append("STATE_HISTORY_CORRUPTION")
   prev=int(e["failed_after"])
  elif e["kind"] in ("RepairCompleted","RestorationCommitted"):
   if int(e["failed_before"])!=prev or int(e["failed_after"])!=prev: problems.append("STATE_HISTORY_LOSS")
   prev=int(e["failed_after"])
 dep=int(js["failures"][0]["target_id"]) if False else 28
 selected=int(ex[0]["after"][0]) if ex else 1
 hit=any((f["target_id"]==selected-1) or (f["target_id"] in [i for i in range(8) if dep&(1<<i)]) for f in fails if f["time_s"] < 7200)
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ex)
 if c_int!=hit: problems.append("OUTCOME_CONTRACT_VIOLATION")
 if x_int!=hit: problems.append("OUTCOME_CONTRACT_VIOLATION")
 if any(e["kind"]=="RestorationCommitted" for e in ex) and (hit or not any(e["kind"]=="RepairCompleted" for e in ex)): problems.append("RESTORATION_PRECONDITION_VIOLATION")
 return {"case":cid,"failures":fails,"contract_hash":hashlib_sha(OUT/"contracts"/f"{cid}.json"),"canonical_events":ce,"external_events":ex,"findings":problems,"status":"ACCEPTED" if not problems else "FAIL"}
def hashlib_sha(p):
 import hashlib; return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 import argparse
 ap=argparse.ArgumentParser(); ap.add_argument("--seed",type=int,default=20261121); a=ap.parse_args()
 random.seed(a.seed); cases=[]
 for i in range(24):
  n=random.randint(0,4); fails=[]
  for fid in range(n): fails.append((random.choice(TIMES),random.choice(list(ASSETS))))
  cid=f"N14-G{i:02d}"; js=make_case(cid,a.seed,fails); cases.append(compare(cid,js))
 bad=[c for c in cases if c["findings"]]
 name="cross-model.json" if a.seed==20261121 else f"cross-model-{a.seed}.json"
 (OUT/name).write_text(json.dumps({"schema":"poie.s5v-n14-generated-cross-model/1","seed":a.seed,"count":len(cases),"accepted":len(cases)-len(bad),"cases":cases},indent=2)+"\n")
 print("generated",len(cases),"accepted",len(cases)-len(bad),"failed",len(bad))
 for c in bad[:5]: print(c["case"],c["findings"])
 return 0 if not bad else 1
if __name__=="__main__": raise SystemExit(main())

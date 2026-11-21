#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json
from pathlib import Path
P=Path(__file__).resolve().parent
CASES=["N14-T00","N14-T01","N14-T02","N14-T03","N14-T04"]
def main():
 results=[]
 for case in CASES:
  spec={}
  for line in (P/"contracts"/f"{case}.contract").read_text().splitlines(): k,v=line.split("=",1); spec[k]=float(v)
  rows=list(csv.DictReader((P/"external"/f"{case}.csv").open())); last=rows[-1]
  j=json.loads((P/"external"/f"{case}.csv.journal.json").read_text())
  problems=[]; expected=[]
  for i in range(int(spec.get("failureCount",0))):
   pre="failure" if i==0 else f"f{i+1}"; kind=int(spec[f"{pre}Kind"]); t=float(spec[f"{pre}Time_s"])
   if kind==1: name=f"robot-m{int(spec[f'{pre}Index'])}"; tid=int(spec[f'{pre}Index'])-1
   elif kind==2: name="tool-t1"; tid=7
   else: tid=int(spec[f"{pre}AssetId"]); name={3:"bus-1",5:"sensor-x",6:"bus-2"}[tid]
   expected.append((t,name,tid))
  expected.sort(key=lambda x:x[0])
  xf=[e for e in j["events"] if e["kind"]=="AssetFailure"]
  if len(xf)>len(expected): problems.append("EVENT_EXTRA")
  for i,e in enumerate(xf):
   if i>=len(expected): break
   if abs(e["at"]-expected[i][0])>1e-9: problems.append("CONTRACT_EVENT_TIME_VIOLATION")
   if e["target"]!=expected[i][1] or int(e["target_id"])!=expected[i][2]: problems.append("EVENT_TARGET_IDENTITY_MISMATCH")
   if int(e.get("failure_id",-1))!=i: problems.append("FAILURE_IDENTITY_MISMATCH")
  prev=0
  for e in j["events"]:
   if e["kind"]=="AssetFailure":
    b=int(e["failed_before"]); a=int(e["failed_after"]); tid=int(e["target_id"])
    if b!=prev or b & ~a: problems.append("STATE_HISTORY_LOSS")
    if not (a & (1<<tid)) or a & ~(b | (1<<tid)): problems.append("STATE_HISTORY_CORRUPTION")
    prev=a
   elif e["kind"] in ("RepairCompleted","RestorationCommitted"):
    if int(e["failed_before"])!=prev or int(e["failed_after"])!=prev: problems.append("STATE_HISTORY_LOSS")
    prev=int(e["failed_after"])
  # per-task resource eligibility from actual harness CSV
  for i in range(int(spec["taskCount"])):
   mask=int(spec[f"t{i+1}RobotMask"]); robot=int(float(last[f"t{i+1}Robot"]))
   if robot<=0 or not (mask & (1<<(robot-1))): problems.append("RESOURCE_ELIGIBILITY_VIOLATION")
  if abs(float(last["makespan_s"])-10800.0)>1e-9 and case!="N14-T03": problems.append("GROUP_MAKESPAN_MISMATCH")
  if any(e["kind"]=="RestorationCommitted" for e in j["events"]) and any(e["kind"]=="RepairInterrupted" for e in j["events"]): problems.append("RESTORATION_PRECONDITION_VIOLATION")
  if any(e["kind"]=="AssetFailure" for e in j["events"] if e["at"]>float(last["time_s"])): problems.append("POST_TERMINATION_EVENT")
  results.append({"case":case,"task_count":int(spec["taskCount"]),"failure_count":len(expected),"applied":len(xf),"resources":{"t1":int(float(last["t1Robot"])),"t2":int(float(last["t2Robot"]))},"makespan":float(last["makespan_s"]),"interrupted":any(e["kind"]=="RepairInterrupted" for e in j["events"]),"problems":problems,"status":"ACCEPTED" if not problems else "FAIL"})
 (P/"trajectory-comparisons/composition.json").write_text(json.dumps({"schema":"poie.s5v-n14-composition/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["applied"],r["resources"],r["problems"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

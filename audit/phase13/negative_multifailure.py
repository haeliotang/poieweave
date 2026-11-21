#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,importlib.util,json
from pathlib import Path
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("n13",P/"validate_multifailure.py"); n13=importlib.util.module_from_spec(spec); spec.loader.exec_module(n13)
results=[]
def mutate(case,fn):
 path=P/"external"/f"N13-{case}.csv.journal.json"; backup=path.read_text(); data=json.loads(backup); fn(data)
 try:
  path.write_text(json.dumps(data)); return n13.compare(case)
 finally: path.write_text(backup)
def record(name,expected,result):
 detected=any(expected in f["class"] for f in result["findings"])
 results.append({"control":name,"expected":expected,"detected":detected,"findings":result["findings"]})
 print(name,"PASS" if detected else "FAIL",result["findings"])
def drop_second(data): data["events"]=[e for e in data["events"] if not (e["kind"]=="AssetFailure" and e.get("failure_id")==1)]
record("second-failure-dropped","EVENT_MISSING",mutate("N13-02",drop_second))
def duplicate(data):
 f=[e for e in data["events"] if e["kind"]=="AssetFailure"][1]; data["events"].insert(data["events"].index(f)+1,copy.deepcopy(f))
record("duplicate-failure","EVENT_EXTRA",mutate("N13-02",duplicate))
def wrong_target(data):
 f=[e for e in data["events"] if e["kind"]=="AssetFailure"][1]; f["target"]="sensor-x"; f["target_id"]=5
record("wrong-target","EVENT_TARGET_IDENTITY_MISMATCH",mutate("N13-02",wrong_target))
def history(data):
 f=[e for e in data["events"] if e["kind"]=="AssetFailure"][1]; f["failed_before"]=0
record("state-history-loss","STATE_HISTORY_LOSS",mutate("N13-02",history))
def post_term(data):
 f=copy.deepcopy([e for e in data["events"] if e["kind"]=="AssetFailure"][0]); f["failure_id"]=1; f["target"]="sensor-x"; f["target_id"]=5; data["events"].append(f)
record("post-termination-event","EVENT_EXTRA",mutate("N13-06",post_term))
def premature(data):
 r=[e for e in data["events"] if e["kind"]=="RestorationCommitted"][0]; r["before"][4]=0
record("premature-restoration","RESTORATION_PRECONDITION_VIOLATION",mutate("N13-00",premature))
# Minimize the wrong-target counterexample to one failure event.
def minimal(data):
 data["events"]=[e for e in data["events"] if e["kind"] in ("InitialRepairStarted","AssetFailure")][:2]
 f=[e for e in data["events"] if e["kind"]=="AssetFailure"][0]; f["target"]="robot-m1"; f["target_id"]=0
returned=mutate("N13-01",minimal)
(P/"minimized-counterexamples/wrong-target.json").write_text(json.dumps({"schema":"poie.s5v-n13-counterexample/1","case":"N13-01","mutation":"AssetFailure target bus/robot -> robot-m1","expected":"EVENT_TARGET_IDENTITY_MISMATCH","result":returned},indent=2)+"\n")
# Both models record the same wrong target: agreement is not correctness.
cpath=P/"canonical/N13-02-events.jsonl"; cbak=cpath.read_text(); rows=[json.loads(x) for x in cbak.splitlines()]
try:
 f=next(e for e in rows if e["event_kind"]=="AssetFailure"); f["target_asset"]="robot-m1"; f["asset_statuses_before"]["robot-m1"]="Available"; f["asset_statuses_after"]["robot-m1"]="Failed"
 cpath.write_text('\n'.join(json.dumps(x) for x in rows)+'\n')
 result=mutate("N13-02",lambda data: (lambda f: (f.__setitem__("target","robot-m1"),f.__setitem__("target_id",0)))(next(e for e in data["events"] if e["kind"]=="AssetFailure")))
 record("both-models-invalid-target","EVENT_TARGET_IDENTITY_MISMATCH",result)
finally: cpath.write_text(cbak)
out=P/"negative-controls/multifailure-results.json"; out.write_text(json.dumps({"schema":"poie.s5v-n13-multifailure-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,importlib.util,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("g",P/"generate_cross_model.py"); g=importlib.util.module_from_spec(spec); spec.loader.exec_module(g)
results=[]
def mutate(case,fn):
 cp=P/"generated-cases/canonical"/f"{case}.jsonl"; ep=P/"generated-cases/external"/f"{case}.csv.journal.json"
 cb=cp.read_text(); eb=ep.read_text(); ce=[json.loads(x) for x in cb.splitlines()]; ex=json.loads(eb); fn(ce,ex)
 try:
  cp.write_text('\n'.join(json.dumps(x) for x in ce)+'\n'); ep.write_text(json.dumps(ex))
  js=json.loads((P/"generated-cases/contracts"/f"{case}.json").read_text()); return g.compare(case,js)
 finally: cp.write_text(cb); ep.write_text(eb)
def record(name,expected,r):
 detected=any(expected in f for f in r["findings"]); results.append({"control":name,"expected":expected,"detected":detected,"findings":r["findings"]})
 print(name,"PASS" if detected else "FAIL",r["findings"])
def pick_case(min_failures=2):
 for path in sorted((P/"generated-cases/external").glob("N14-G*.csv.journal.json")):
  j=json.loads(path.read_text())
  if sum(1 for e in j["events"] if e["kind"]=="AssetFailure")>=min_failures: return path.name.split(".")[0]
 raise SystemExit("no multi-failure generated case")
MULTI=pick_case()
def both_time(ce,ex):
 for e in ce:
  if e["event_kind"]=="AssetFailure": e["at_s"]+=100
 for e in ex["events"]:
  if e["kind"]=="AssetFailure": e["at"]+=100
record("shared-wrong-time","CONTRACT_EVENT_TIME_VIOLATION",mutate("N14-G03",both_time))
def missing(ce,ex):
 ex["events"]=[e for e in ex["events"] if not (e["kind"]=="AssetFailure" and e.get("failure_id")==1)]
record("second-failure-missing","EVENT_MISSING",mutate(MULTI,missing))
def history(ce,ex):
 f=[e for e in ex["events"] if e["kind"]=="AssetFailure"][-1]; f["failed_before"]=0
record("state-history-loss","STATE_HISTORY_LOSS",mutate(MULTI,history))
def id_swap(ce,ex):
 fs=[e for e in ex["events"] if e["kind"]=="AssetFailure"]
 if len(fs)>1: fs[0]["failure_id"],fs[1]["failure_id"]=fs[1]["failure_id"],fs[0]["failure_id"]
record("failure-id-swap","FAILURE_IDENTITY_MISMATCH",mutate(MULTI,id_swap))
def post_term(ce,ex):
 f=copy.deepcopy([e for e in ex["events"] if e["kind"]=="AssetFailure"][0]); f["failure_id"]=3; f["target"]="sensor-x"; f["target_id"]=5; ex["events"].append(f)
record("post-termination-event","EVENT_EXTRA",mutate("N14-G02",post_term))
# Task-2 resource violation through the composition validator's CSV.
csv=P/"external/N14-T00.csv"; bak=csv.read_text()
try:
 lines=bak.splitlines(); h=lines[0].split(','); idx=h.index('t2Robot'); last=lines[-1].split(','); last[idx]='1'; lines[-1]=','.join(last); csv.write_text('\n'.join(lines)+'\n')
 r=subprocess.run([sys.executable,str(P/"validate_composition.py")],capture_output=True,text=True)
 detected=r.returncode!=0 and 'RESOURCE_ELIGIBILITY_VIOLATION' in r.stdout
 results.append({"control":"task2-resource","expected":"RESOURCE_ELIGIBILITY_VIOLATION","detected":detected,"findings":r.stdout.strip().splitlines()[-1:]})
 print("task2-resource","PASS" if detected else "FAIL")
finally: csv.write_text(bak)
# Invalid failure count rejects at FMI initialization.
bad=P/"contracts/N14-invalid-count.contract"; bad.write_text((P/"contracts/N14-T00.contract").read_text().replace('failureCount=0','failureCount=5'))
r=subprocess.run(["external/fmu-n14-scheduler/build/harness","external/fmu-n14-scheduler/poie-n14-scheduler.fmu","900",str(bad),str(P/"external/N14-invalid-count.csv")],capture_output=True,text=True)
detected=r.returncode!=0
results.append({"control":"failure-count-capacity","expected":"INVALID_FAILURE_COUNT","detected":detected,"findings":[r.stderr.strip() or r.stdout.strip()]})
print("failure-count-capacity","PASS" if detected else "FAIL")
subprocess.run([sys.executable,str(P/"validate_composition.py")],check=True,capture_output=True)
(P/"negative-controls/cross-model-results.json").write_text(json.dumps({"schema":"poie.s5v-n14-negative/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(x["detected"] for x in results) else 1)

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,importlib.util,json
from pathlib import Path
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("g",P/"generate_group_cases.py"); g=importlib.util.module_from_spec(spec); spec.loader.exec_module(g)
results=[]
def mutate(case,tasks,failures,fn):
 cp=P/"canonical"/f"{case}.jsonl"; ep=P/"external"/f"{case}.csv"; jp=P/"external"/f"{case}.csv.journal.json"
 cb=cp.read_text(); eb=ep.read_text(); jb=jp.read_text(); ce=[json.loads(x) for x in cb.splitlines()]; rows=list(__import__("csv").DictReader(eb.splitlines())); ext=json.loads(jb)
 fn(ce,rows,ext)
 try:
  cp.write_text('\n'.join(json.dumps(x) for x in ce)+'\n')
  import csv,io
  out=io.StringIO(); w=csv.DictWriter(out,fieldnames=rows[0].keys()); w.writeheader(); w.writerows(rows); ep.write_text(out.getvalue())
  jp.write_text(json.dumps(ext)); return g.compare(case,tasks,failures)
 finally: cp.write_text(cb); ep.write_text(eb); jp.write_text(jb)
def record(name,expected,r):
 detected=any(expected in p for p in r["problems"]); results.append({"control":name,"expected":expected,"detected":detected,"problems":r["problems"]})
 print(name,"PASS" if detected else "FAIL",r["problems"])
tasks=[(7200,1,False),(10800,2,False)]
# N14R-00 has no failures.
record("task-missing","TASK_COUNT",mutate("N14R-00",tasks,[],lambda ce,rows,ext: ce[0]["running_repair"]["task_resources"].pop()))
record("task-start-shift","TASK_START_0",mutate("N14R-00",tasks,[],lambda ce,rows,ext: rows[-1].__setitem__("t1Start_s","100.0")))
record("task-resource","TASK_RESOURCE_0",mutate("N14R-00",tasks,[],lambda ce,rows,ext: rows[-1].__setitem__("t1Robot","2.0")))
record("early-restoration","RESTORATION_TIME",mutate("N14R-00",tasks,[],lambda ce,rows,ext: ext["events"].insert(-1,{"kind":"RestorationCommitted","at":7200.0,"before":[0,1,0,0,1,1,0],"after":[0,1,0,0,1,1,0],"target":"a-lathe"})))
# Shared wrong event time in N14R-02.
def shift(ce,rows,ext):
 for e in ce:
  if e["event_kind"]=="AssetFailure": e["at_s"]+=100
 for e in ext["events"]:
  if e["kind"]=="AssetFailure": e["at"]+=100
record("shared-wrong-time","EVENT_TIME",mutate("N14R-02",[(7200,1,False),(10800,2,False)],[(3600,5),(5400,6)],shift))
(P/"negative-controls/group-results.json").write_text(json.dumps({"schema":"poie.s5v-n14r-negative/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

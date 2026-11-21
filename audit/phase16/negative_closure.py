#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
def mutate(case,fn,expected):
 cp=P/"canonical"/f"{case}.jsonl"; ep=P/"external"/f"{case}.csv.journal.json"
 cb=cp.read_text(); eb=ep.read_text()
 try:
  ce=[json.loads(x) for x in cb.splitlines()]; doc=json.loads(eb); ev=doc["events"]; fn(ce,ev)
  cp.write_text('\n'.join(json.dumps(x) for x in ce)+'\n'); doc["events"]=ev; ep.write_text(json.dumps(doc))
  subprocess.run([sys.executable,str(P/"validate_closure.py")],capture_output=True,text=True)
  data=json.loads((P/"dependency-trajectories/results.json").read_text()); row=next(x for x in data["cases"] if x["case"]==case)
  return {"control":expected,"case":case,"expected":expected,"detected":any(expected in f for f in row["findings"]),"findings":row["findings"]}
 finally: cp.write_text(cb); ep.write_text(eb)
def main():
 results=[]
 def both_remove(ce,ev):
  for e in ce:
   if e.get("running_repair") and e["at_s"]>=7200-1e-9:
    e["running_repair"]["execution_dependencies"]=[d for d in e["running_repair"]["execution_dependencies"] if d!="bus-1"]
  for e in ev:
   if e.get("at",-1)>=7200-1e-9 and "dep_after" in e: e["dep_after"]&=~(1<<3)
 results.append(mutate("N16-06",both_remove,"SHARED_DEPENDENCY_LOSS"))
 def wrong_target(ce,ev):
  for e in ev:
   if e["kind"]=="DependencyDeactivated" and e.get("target")=="bus-1": e["target"]="bus-2"
 results.append(mutate("N16-04",wrong_target,"EVENT_SET_DIVERGENCE"))
 def extra_event(ce,ev):
  src=next(e for e in ev if e["kind"]=="DependencyDeactivated")
  extra=dict(src); extra["at"]=src["at"]; ev.insert(ev.index(src)+1,extra)
 results.append(mutate("N16-04",extra_event,"EVENT_SET_DIVERGENCE"))
 def stale_hit(ce,ev):
  ev.append({"kind":"RepairInterrupted","at":7201.0,"target":"a-lathe","origin":"NATIVE_EXTERNAL_FMU"})
 results.append(mutate("N16-04",stale_hit,"EVENT_SET_DIVERGENCE"))
 (P/"negative-controls").mkdir(exist_ok=True)
 (P/"negative-controls/results.json").write_text(json.dumps({"schema":"poie.s5v-n16-negative/1","results":results},indent=2)+"\n")
 for r in results: print(r["expected"],"PASS" if r["detected"] else "FAIL",r["findings"])
 return 0 if all(r["detected"] for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

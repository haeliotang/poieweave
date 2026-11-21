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
  ce=[json.loads(x) for x in cb.splitlines()]; journal=json.loads(eb)["events"]; fn(ce,journal)
  cp.write_text('\n'.join(json.dumps(x) for x in ce)+'\n'); ep.write_text(json.dumps({"events":journal}))
  r=subprocess.run([sys.executable,str(P/"validate_release.py")],capture_output=True,text=True)
  data=json.loads((P/"resource-lifetimes/results.json").read_text())
  row=next(x for x in data["cases"] if x["case"]==case); detected=any(expected in f for f in row["findings"])
  return {"control":expected,"case":case,"expected":expected,"detected":detected,"findings":row["findings"]}
 finally: cp.write_text(cb); ep.write_text(eb)
def main():
 results=[]
 def missing(ce,j): ce[:]=[e for e in ce if not (e["event_kind"]=="ResourceReleased" and e["target_asset"]=="robot-m1")]
 results.append(mutate("N15-D1-R1-7201",missing,"RESOURCE_RELEASE_MISSING_robot-m1"))
 def early(ce,j):
  for e in ce:
   if e["event_kind"]=="ResourceReleased" and e["target_asset"]=="robot-m1": e["at_s"]=7300.0
 results.append(mutate("N15-D1-R1-7201",early,"RESOURCE_RELEASE_TIME_robot-m1"))
 def shared(ce,j):
  ce.append({"event_kind":"ResourceReleased","at_s":7200.0,"target_asset":"robot-m1","event_sequence":99,"event_batch_sequence":99,"asset_statuses_before":{},"asset_statuses_after":{},"inventory_before":{},"inventory_after":{},"running_repair":None})
 results.append(mutate("N15-D2-SERIAL-7201",shared,"SHARED_RESOURCE_RELEASE_VIOLATION_robot-m1"))
 def false_hit(ce,j):
  ce.append({"event_kind":"RepairInterrupted","at_s":7201.0,"target_asset":"repair","event_sequence":99,"event_batch_sequence":99,"asset_statuses_before":{},"asset_statuses_after":{},"inventory_before":{},"inventory_after":{},"running_repair":None})
 results.append(mutate("N15-D1-R1-7201",false_hit,"CANONICAL_FAILURE_SENSITIVITY"))
 def add_complete(ce,j):
  ce.append({"event_kind":"RepairCompleted","at_s":7200.0,"target_asset":"n14-group","event_sequence":99,"event_batch_sequence":99,"asset_statuses_before":{},"asset_statuses_after":{},"inventory_before":{},"inventory_after":{},"running_repair":None})
 results.append(mutate("N15-OR-RELEASE",add_complete,"PREMATURE_GROUP_COMMIT"))
 def premature_no_fail(ce,j):
  ce.append({"event_kind":"ResourceReleased","at_s":7200.0,"target_asset":"robot-m1","event_sequence":99,"event_batch_sequence":99,"asset_statuses_before":{},"asset_statuses_after":{},"inventory_before":{},"inventory_after":{},"running_repair":None})
 results.append(mutate("N15-D2-SERIAL-NOFAIL",premature_no_fail,"SHARED_RESOURCE_RELEASE_VIOLATION_robot-m1"))
 (P/"negative-controls").mkdir(exist_ok=True)
 (P/"negative-controls/release-results.json").write_text(json.dumps({"schema":"poie.s5v-n15-negative/1","results":results},indent=2)+"\n")
 for r in results: print(r["expected"],"PASS" if r["detected"] else "FAIL",r["findings"])
 return 0 if all(r["detected"] for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

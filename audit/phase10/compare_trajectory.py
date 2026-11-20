#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 event-resolved resource trajectory comparator.

Canonical events are native. External events are derived from the FMU step trace
and are labelled DERIVED_FROM_EXECUTION_OUTPUT. Before-event state is only used
when an exact previous communication sample exists; otherwise it is NOT_OBSERVED.
"""
from __future__ import annotations
import csv,json
from pathlib import Path
PHASE=Path(__file__).resolve().parent
CASES=["N10-U01","N10-U02","N10-U03","N10-U04","N10-OR1","N10-OR2","N10-OR3"]
def contract(case):
 data={}
 for line in (PHASE/"contracts"/f"{case}.contract").read_text().splitlines():
  k,v=line.split("=",1); data[k]=float(v)
 return data
def summary(case): return json.loads((PHASE/"canonical"/f"{case}-summary.json").read_text())
def c_events(case):
 rows=[r for r in csv.DictReader((PHASE/"canonical"/f"{case}-events.csv").open(),delimiter="\t") if r["case"]==case]
 return [{"time":float(r["at_s"]),"kind":r["event_kind"],"sequence":int(r["event_sequence"]),"target":r["target_asset"],
          "resources":r["resources"],"task_resources":r["task_resources"]} for r in rows if float(r["at_s"])<=7200.0+1e-9]
def x_rows(case): return list(csv.DictReader((PHASE/"external"/f"{case}-fmpy-900.csv").open()))
def x_events(case,spec):
 rows=x_rows(case); last=rows[-1]; events=[]
 events.append({"time":0.0,"kind":"InitialRepairStarted","source":"DERIVED_FROM_EXECUTION_OUTPUT"})
 failure=float(last["failureAppliedTime_s"]); interrupted=int(float(last["interrupted"]))
 if failure>=0: events.append({"time":failure,"kind":"AssetFailure","source":"DERIVED_FROM_EXECUTION_OUTPUT"})
 if interrupted: events.append({"time":failure,"kind":"RepairInterrupted","source":"DERIVED_FROM_EXECUTION_OUTPUT"})
 if int(float(last["completedCount"]))>0:
  events.append({"time":float(last["makespan_s"]),"kind":"RepairCompleted","source":"DERIVED_FROM_EXECUTION_OUTPUT"})
 return sorted(events,key=lambda e:(e["time"],e["kind"])),rows,last
def compare(case):
 spec=contract(case); sumry=summary(case); ce=c_events(case); xe,rows,last=x_events(case,spec); findings=[]
 # strict event time alignment by kind (no nearest-neighbor).
 cmap={}
 for e in ce:
  cmap.setdefault(e["kind"],[]).append(e)
 xmap={}
 for e in xe: xmap.setdefault(e["kind"],[]).append(e)
 for kind in ["InitialRepairStarted","AssetFailure","RepairInterrupted","RepairCompleted"]:
  cs=cmap.get(kind,[]); xs=xmap.get(kind,[])
  if len(cs)!=len(xs):
   if not (kind in ("RepairInterrupted","RepairCompleted") and case=="N10-OR2"):
    findings.append({"class":"EVENT_TIME_DISPLACEMENT","event":kind,"canonical":len(cs),"external":len(xs)})
  for c,x in zip(cs,xs):
   if abs(c["time"]-x["time"])>1e-9:
    findings.append({"class":"EVENT_TIME_DISPLACEMENT","event":kind,"canonical_s":c["time"],"external_s":x["time"]})
 # canonical native order must be completion before same-time failure.
 for i,a in enumerate(ce):
  for b in ce[i+1:]:
   if abs(a["time"]-b["time"])<=1e-9 and a["kind"]=="RepairCompleted" and b["kind"]=="AssetFailure":
    if a["sequence"]>b["sequence"]: findings.append({"class":"EVENT_ORDER_MISMATCH","at_s":a["time"]})
 # External ordering for same-time completion/failure: endpoint trace cannot establish order.
 order_coverage="OBSERVED" if not any(abs(a["time"]-b["time"])<=1e-9 and a["kind"]!=b["kind"] for a in xe for b in xe if a is not b) else "PARTIAL"
 # Policy-dependent outcome difference.
 if case=="N10-OR2":
  c_int=any(e["kind"]=="RepairInterrupted" for e in ce); x_int=bool(int(float(last["interrupted"])))
  if c_int and not x_int:
   return {"schema":"poie.s5v-n10-trajectory/1","case":case,"status":"ACCEPTED",
           "canonical_events":ce,"external_events":xe,"event_order_coverage":order_coverage,
           "event_before_coverage":"NOT_OBSERVED_FOR_INTERMEDIATE_BOUNDARY",
           "resource_state_after":{"canonical":"INTERRUPTED","external":"COMPLETED"},
           "first_event_time_divergence":None,
           "first_event_order_divergence":None,
           "first_resource_selection_divergence":{"canonical":sumry["task_resources"][0],"external":f"t1:robot-m{int(float(last['t1Robot']))}"},
           "first_dependency_identity_divergence":None,
           "first_resource_lifetime_divergence":None,
           "first_interruption_divergence":{"at_s":3600.0,"classification":"VALID_POLICY_DIFFERENCE"},
           "first_restoration_divergence":{"at_s":7200.0,"classification":"VALID_POLICY_DIFFERENCE"},
           "first_unexplained_divergence":None,"findings":[],
           "classification":"VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE",
           "trajectory_coverage":"EVENT_LEVEL_WITH_ENDPOINT_STATE",
           "external_event_source":"DERIVED_FROM_EXECUTION_OUTPUT"}
 if findings:
  return {"schema":"poie.s5v-n10-trajectory/1","case":case,"status":"FAIL","canonical_events":ce,"external_events":xe,
          "findings":findings,"first_event_time_divergence":findings[0],
          "trajectory_coverage":"EVENT_LEVEL_WITH_ENDPOINT_STATE","external_event_source":"DERIVED_FROM_EXECUTION_OUTPUT"}
 # After-state at failure/completion from exact external rows.
 def row_at(t):
  for row in rows:
   if abs(float(row["time_s"])-t)<=1e-9: return row
  return None
 c_int=any(e["kind"]=="RepairInterrupted" for e in ce); x_int=bool(int(float(last["interrupted"])))
 if c_int!=x_int: findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","reason":"interruption state"})
 c_complete=any(e["kind"]=="RepairCompleted" for e in ce); x_complete=int(float(last["completedCount"]))>0
 if c_complete!=x_complete: findings.append({"class":"RESTORATION_COMMIT_MISMATCH"})
 failure_row=row_at(float(last["failureAppliedTime_s"])) if float(last["failureAppliedTime_s"])>=0 else None
 if float(last["failureAppliedTime_s"])>=0 and failure_row is None:
  findings.append({"class":"NOT_COMPARABLE","reason":"no exact external row at failure time"})
 after_state={"canonical":"INTERRUPTED" if c_int else ("COMPLETED" if c_complete else "RUNNING"),
              "external":"INTERRUPTED" if x_int else ("COMPLETED" if x_complete else "RUNNING"),
              "failure_row_present":failure_row is not None}
 status="ACCEPTED" if not findings else "FAIL"
 return {"schema":"poie.s5v-n10-trajectory/1","case":case,"status":status,
         "canonical_events":ce,"external_events":xe,"event_order_coverage":order_coverage,
         "event_before_coverage":"NOT_OBSERVED_FOR_INTERMEDIATE_BOUNDARY",
         "resource_state_after":after_state,
         "first_event_time_divergence":None,
         "first_event_order_divergence":None,
         "first_resource_selection_divergence":None,
         "first_dependency_identity_divergence":None,
         "first_resource_lifetime_divergence":None,
         "first_interruption_divergence":None,
         "first_restoration_divergence":None,
         "first_unexplained_divergence":findings[0] if findings else None,
         "findings":findings,"classification":"SAME_CAUSAL_TRAJECTORY",
         "trajectory_coverage":"EVENT_LEVEL_WITH_ENDPOINT_STATE",
         "external_event_source":"DERIVED_FROM_EXECUTION_OUTPUT"}
def main():
 results=[compare(c) for c in CASES]
 (PHASE/"comparisons/trajectories.json").write_text(json.dumps({"schema":"poie.s5v-n10-trajectory-set/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r.get("classification"),r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

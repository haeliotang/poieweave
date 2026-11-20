#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 trajectory mutation controls."""
from __future__ import annotations
import csv,importlib.util,json
from pathlib import Path
PHASE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("n10t",PHASE/"compare_trajectory.py"); n10=importlib.util.module_from_spec(spec); spec.loader.exec_module(n10)
results=[]
def read_csv(path):
 rows=list(csv.DictReader(path.open())); return list(rows[0].keys()),rows
def write_csv(path,fields,rows):
 with path.open("w",newline="") as h:
  w=csv.DictWriter(h,fieldnames=fields); w.writeheader(); w.writerows(rows)
def mutate_external(case,fn):
 path=PHASE/"external"/f"{case}-fmpy-900.csv"; fields,rows=read_csv(path); backup=path.read_text()
 try:
  fn(rows); write_csv(path,fields,rows); return n10.compare(case)
 finally: path.write_text(backup)
def record(name,expected,result):
 classes={f["class"] for f in result["findings"]}; detected=expected in classes
 results.append({"control":name,"expected":expected,"detected":detected,"classes":sorted(classes)})
 print(name,"PASS" if detected else "FAIL",sorted(classes))
def add(name,expected,case,fn): record(name,expected,mutate_external(case,fn))
add("failure-time-shift","EVENT_TIME_DISPLACEMENT","N10-U01",lambda rows: rows[-1].update(failureAppliedTime_s="3601.0"))
add("exact-failure-row-removed","EVENT_TIME_DISPLACEMENT","N10-U01",lambda rows: rows.pop(4))
add("interruption-state-mismatch","EVENT_TIME_DISPLACEMENT","N10-U01",lambda rows: rows[-1].update(interrupted="0.0",completedCount="1.0"))
# Same-time canonical order swap in U04.
path=PHASE/"canonical/N10-U04-events.csv"; backup=path.read_text(); rows=list(csv.DictReader(backup.splitlines(),delimiter="\t"))
try:
 for row in rows:
  if row["event_kind"]=="RepairCompleted": row["event_sequence"]="4"
  elif row["event_kind"]=="AssetFailure": row["event_sequence"]="2"
 fields=list(rows[0].keys())
 with path.open("w",newline="") as h:
  w=csv.DictWriter(h,fieldnames=fields,delimiter="\t"); w.writeheader(); w.writerows(rows)
 result=n10.compare("N10-U04")
finally: path.write_text(backup)
record("same-time-order-swap","EVENT_ORDER_MISMATCH",result)
# Sanity: legal policy-dependent difference remains accepted.
result=n10.compare("N10-OR2")
detected=result["status"]=="ACCEPTED" and result["classification"]=="VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE"
results.append({"control":"policy-difference-not-failure","expected":"ACCEPTED_POLICY_DIFFERENCE","detected":detected,"classes":[]})
print("policy-difference-not-failure","PASS" if detected else "FAIL")
output=PHASE/"negative-controls/trajectory-results.json"
output.write_text(json.dumps({"schema":"poie.s5v-n10-trajectory-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 named-upstream and OR decision controls."""
from __future__ import annotations
import csv,importlib.util,json
from pathlib import Path
PHASE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("n10c",PHASE/"compare_causal.py"); n10=importlib.util.module_from_spec(spec); spec.loader.exec_module(n10)
results=[]
def read_csv(path):
 rows=list(csv.DictReader(path.open())); return list(rows[0].keys()),rows
def write_csv(path,fields,rows):
 with path.open("w",newline="") as h:
  w=csv.DictWriter(h,fieldnames=fields); w.writeheader(); w.writerows(rows)
def mutate_external(case,fn):
 path=PHASE/"external"/f"{case}-fmpy-900.csv"; fields,rows=read_csv(path); backup=path.read_text()
 try:
  fn(rows[-1]); write_csv(path,fields,rows); return n10.compare(case)
 finally: path.write_text(backup)
def record(name,expected,result):
 classes={f["class"] for f in result["findings"]}; detected=expected in classes
 results.append({"control":name,"expected":expected,"detected":detected,"classes":sorted(classes)})
 print(name,"PASS" if detected else "FAIL",sorted(classes))
def add(name,expected,case,fn): record(name,expected,mutate_external(case,fn))
add("active-bus-ignored","UPSTREAM_DEPENDENCY_MISMATCH","N10-U01",lambda r:r.update(interrupted="0.0",completedCount="1.0",restored1="1.0"))
add("unrelated-upstream-interrupts","UPSTREAM_DEPENDENCY_MISMATCH","N10-U02",lambda r:r.update(interrupted="1.0",completedCount="0.0",restored1="0.0"))
add("wrong-failure-target-id","UPSTREAM_DEPENDENCY_MISMATCH","N10-U01",lambda r:r.update(failureTargetId="4.0"))
add("missing-dependency-path","UPSTREAM_DEPENDENCY_MISMATCH","N10-U01",lambda r:r.update(dependencyPathCode="0.0"))
add("unselected-or-path-interrupts","EXECUTION_DEPENDENCY_MISMATCH","N10-OR2",lambda r:r.update(interrupted="1.0",completedCount="0.0",restored1="0.0",makespan_s="3600.0"))
add("or-tool-and-ignored","EXECUTION_DEPENDENCY_MISMATCH","N10-OR3",lambda r:r.update(toolTaskMask="0.0"))
add("or-robot-outside-mask","RESOURCE_ELIGIBILITY_VIOLATION","N10-OR1",lambda r:r.update(t1Robot="3.0"))
# Both models choose a robot outside the task mask.
summary_path=PHASE/"canonical/N10-OR1-summary.json"; backup=summary_path.read_text()
try:
 import re
 mutated=backup.replace('"0:robot-m1"','"0:robot-m3"')
 summary_path.write_text(mutated)
 result=mutate_external("N10-OR1",lambda r:r.update(t1Robot="3.0"))
 record("both-or-models-invalid","RESOURCE_ELIGIBILITY_VIOLATION",result)
finally: summary_path.write_text(backup)
# A legal different OR decision must not be reported as a contract violation.
result=n10.compare("N10-OR1")
detected = result["status"]=="ACCEPTED" and result["resource_decision"]=="DIFFERENT_BUT_VALID_DECISION"
results.append({"control":"valid-or-difference-not-failure","expected":"ACCEPTED_DIFFERENT_VALID","detected":detected,"classes":[]})
print("valid-or-difference-not-failure","PASS" if detected else "FAIL")
output=PHASE/"negative-controls/causal-results.json"
output.write_text(json.dumps({"schema":"poie.s5v-n10-causal-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

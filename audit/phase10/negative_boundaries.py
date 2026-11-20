#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 boundary mutation controls."""
from __future__ import annotations
import csv,importlib.util,json
from pathlib import Path
PHASE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("n10",PHASE/"compare_boundaries.py"); n10=importlib.util.module_from_spec(spec); spec.loader.exec_module(n10)
results=[]
def read_csv(path):
    rows=list(csv.DictReader(path.open())); return list(rows[0].keys()),rows
def write_csv(path,fields,rows):
    with path.open("w",newline="") as h:
        w=csv.DictWriter(h,fieldnames=fields); w.writeheader(); w.writerows(rows)
def mutate_external(case,fn):
    path=PHASE/"external"/f"{case}-fmpy-1.csv"; fields,rows=read_csv(path); backup=path.read_text()
    try:
        fn(rows[-1]); write_csv(path,fields,rows); return n10.compare(case)
    finally: path.write_text(backup)
def record(name,expected,result):
    classes={f["class"] for f in result["findings"]}; detected=expected in classes
    results.append({"control":name,"expected":expected,"detected":detected,"classes":sorted(classes)})
    print(name,"PASS" if detected else "FAIL",sorted(classes))
def add(name,expected,case,fn): record(name,expected,mutate_external(case,fn))
add("7199-failure-shifted-to-7200","EVENT_TIME_DISPLACEMENT","N10-01",lambda r:r.update(failureAppliedTime_s="7200.0"))
add("horizon-failure-wrongly-applied","OBSERVATION_ARTIFACT","N10-06",lambda r:r.update(failureAppliedTime_s="7200.0"))
add("post-commit-failure-interrupts","EXECUTION_DEPENDENCY_MISMATCH","N10-05",lambda r:r.update(interrupted="1.0",completedCount="0.0",makespan_s="12000.0",release_s="12000.0"))
# Same-time completion/failure order mutation in canonical events.
path=PHASE/"canonical/N10-02-events.csv"; backup=path.read_text(); rows=list(csv.DictReader(backup.splitlines(),delimiter="\t"))
try:
    for row in rows:
        if row["event_kind"]=="RepairCompleted": row["event_sequence"]="4"
        elif row["event_kind"]=="AssetFailure": row["event_sequence"]="2"
    fields=list(rows[0].keys())
    with path.open("w",newline="") as h:
        w=csv.DictWriter(h,fieldnames=fields,delimiter="\t"); w.writeheader(); w.writerows(rows)
    result=n10.compare("N10-02")
finally: path.write_text(backup)
record("same-time-order-swap","EVENT_ORDER_MISMATCH",result)
# Requested failure missing from canonical events.
path=PHASE/"canonical/N10-03-events.csv"; backup=path.read_text(); rows=list(csv.DictReader(backup.splitlines(),delimiter="\t"))
try:
    rows=[row for row in rows if row["event_kind"]!="AssetFailure"]; fields=list(rows[0].keys())
    with path.open("w",newline="") as h:
        w=csv.DictWriter(h,fieldnames=fields,delimiter="\t"); w.writeheader(); w.writerows(rows)
    result=n10.compare("N10-03")
finally: path.write_text(backup)
record("failure-not-applied","OBSERVATION_ARTIFACT",result)
output=PHASE/"negative-controls/boundary-results.json"
output.write_text(json.dumps({"schema":"poie.s5v-n10-boundary-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

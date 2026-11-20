#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N11 native journal mutation controls; both harness and fmpy journals are mutated."""
from __future__ import annotations
import copy,importlib.util,json
from pathlib import Path
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("n11",P/"validate_native.py"); n11=importlib.util.module_from_spec(spec); spec.loader.exec_module(n11)
results=[]
def paths(case): return [P/"external"/f"N11-{case}.csv.journal.json", P/"external"/f"N11-{case}-fmpy.csv.journal.json"]
def run(case,mut):
    ps=paths(case); backups=[p.read_text() for p in ps]; base=json.loads(backups[0]); data=copy.deepcopy(base); mut(data)
    for p in ps: p.write_text(json.dumps(data)+"\n")
    try: return n11.validate(case)
    finally:
        for p,b in zip(ps,backups): p.write_text(b)
def record(name,expected,result):
    detected=any(expected in finding for finding in result["findings"])
    results.append({"control":name,"expected":expected,"detected":detected,"findings":result["findings"]})
    print(name,"PASS" if detected else "FAIL",result["findings"])
def delete_failure(data):
    data["events"]=[e for e in data["events"] if e["kind"]!="AssetFailure"]; data["count"]-=1
record("event-missing","event count AssetFailure",run("N10-U01",delete_failure))
def duplicate(data): data["events"].insert(1,copy.deepcopy(data["events"][1])); data["count"]+=1
record("event-extra","event count AssetFailure",run("N10-U01",duplicate))
def swap(data):
    ev=data["events"]; a=next(i for i,e in enumerate(ev) if e["kind"]=="AssetFailure"); b=next(i for i,e in enumerate(ev) if e["kind"]=="RepairInterrupted"); ev[a]["seq"],ev[b]["seq"]=ev[b]["seq"],ev[a]["seq"]
record("same-time-order-swap","same-time order",run("N10-U01",swap))
def shift(data): next(e for e in data["events"] if e["kind"]=="AssetFailure")["at"]+=1.0
record("event-time-shift","event time AssetFailure",run("N10-U01",shift))
def phase(data): e=data["events"][0]; e["before"],e["after"]=e["after"],e["before"]
record("before-after-swap","initial robot state",run("N10-U01",phase))
def overflow(data): data["overflow"]=True
record("journal-overflow","OBSERVATION_OVERFLOW",run("N10-U01",overflow))
def provenance(data): data["events"][0]["origin"]="DERIVED_FROM_EXECUTION_OUTPUT"
record("fake-native-provenance","EVIDENCE_PROVENANCE_VIOLATION",run("N10-U01",provenance))
def restore(data): next(e for e in data["events"] if e["kind"]=="RestorationCommitted")["after"][5]=0
record("restoration-flag","state after RestorationCommitted",run("N10-00",restore))
def dependency(data):
    next(e for e in data["events"] if e["kind"]=="InitialRepairStarted")["after"][2]=0
record("wrong-dependency-witness","RESOURCE_DEPENDENCY_MISMATCH",run("N10-U01",dependency))
output=P/"negative-controls/native-results.json"
output.write_text(json.dumps({"schema":"poie.s5v-n11-native-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

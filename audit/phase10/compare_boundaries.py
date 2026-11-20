#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 strict event-boundary comparator (endpoint external + native canonical events)."""
from __future__ import annotations
import csv, json
from pathlib import Path
PHASE=Path(__file__).resolve().parent
CASES=["N10-00","N10-01","N10-02","N10-03","N10-04","N10-05","N10-06"]
def contract(case):
    data={}
    for line in (PHASE/"contracts"/f"{case}.contract").read_text().splitlines():
        key,value=line.split("=",1); data[key]=float(value)
    return data
def summary(case): return json.loads((PHASE/"canonical"/f"{case}-summary.json").read_text())
def events(case):
    return [row for row in csv.DictReader((PHASE/"canonical"/f"{case}-events.csv").open(),delimiter="\t") if row["case"]==case]
def external(case): return list(csv.DictReader((PHASE/"external"/f"{case}-fmpy-1.csv").open()))[-1]
def compare(case):
    spec=contract(case); sumry=summary(case); ev=events(case); ext=external(case); findings=[]
    completion=sumry["completion_s"]; interruption=sumry["interruption_s"]
    ext_makespan=float(ext["makespan_s"]); ext_interrupted=int(float(ext["interrupted"]))
    ext_failure=float(ext["failureAppliedTime_s"]); requested=spec["failureTime_s"]; horizon=spec["horizon_s"]
    native_failure=next((e for e in ev if e["event_kind"]=="AssetFailure"),None)
    native_completion=next((e for e in ev if e["event_kind"]=="RepairCompleted"),None)
    if interruption is not None and ext_interrupted != 1:
        findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","reason":"canonical interrupted, external continued"})
    if interruption is None and ext_interrupted == 1:
        findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","reason":"external interrupted, canonical continued"})
    if completion is not None and abs(completion-ext_makespan)>1e-9:
        findings.append({"class":"EVENT_TIME_DISPLACEMENT","kind":"completion"})
    if requested >= horizon:
        if native_failure is not None:
            findings.append({"class":"OBSERVATION_ARTIFACT","reason":"horizon failure executed in canonical"})
        if ext_failure >= 0:
            findings.append({"class":"OBSERVATION_ARTIFACT","reason":"horizon failure executed in external"})
    elif requested >= 0:
        if native_failure is not None:
            native_time=float(native_failure["at_s"])
            if abs(native_time-requested)>1e-9:
                findings.append({"class":"EVENT_TIME_DISPLACEMENT","kind":"failure","expected":requested,"actual":native_time})
            if abs(ext_failure-requested)>1e-9:
                findings.append({"class":"EVENT_TIME_DISPLACEMENT","kind":"external_failure","expected":requested,"actual":ext_failure})
        else:
            findings.append({"class":"OBSERVATION_ARTIFACT","reason":"requested failure not observed natively"})
    if native_completion is not None and native_failure is not None and abs(float(native_completion["at_s"])-float(native_failure["at_s"]))<=1e-9:
        if int(native_completion["event_sequence"]) > int(native_failure["event_sequence"]):
            findings.append({"class":"EVENT_ORDER_MISMATCH","reason":"failure precedes completion at same timestamp"})
    status="ACCEPTED" if not findings else "FAIL"
    return {"schema":"poie.s5v-n10-boundary/1","case":case,"contract_sha256":__import__("hashlib").sha256((PHASE/"contracts"/f"{case}.contract").read_bytes()).hexdigest(),
            "canonical":{"completion_s":completion,"interruption_s":interruption,
                         "native_failure_s":float(native_failure["at_s"]) if native_failure else None,
                         "native_failure_sequence":int(native_failure["event_sequence"]) if native_failure else None,
                         "native_completion_sequence":int(native_completion["event_sequence"]) if native_completion else None,
                         "resource_release_observed":False,"provenance":"NATIVE_CANONICAL_RUNTIME"},
            "external":{"makespan_s":ext_makespan,"interrupted":ext_interrupted,"failure_applied_s":ext_failure,
                        "completed_count":int(float(ext["completedCount"])),"provenance":"NATIVE_EXTERNAL_FMU",
                        "trajectory_coverage":"ENDPOINT_ONLY"},
            "requested_failure_s":requested,"horizon_s":horizon,"findings":findings,
            "first_causal_divergence":findings[0] if findings else {"class":"NO_DIVERGENCE","at_s":None},
            "resource_release_claim":"NOT_OBSERVED","status":status}
def main():
    results=[compare(case) for case in CASES]
    (PHASE/"comparisons/boundaries.json").write_text(json.dumps({"schema":"poie.s5v-n10-boundary-set/1","cases":results},indent=2)+"\n")
    for result in results: print(result["case"],result["status"],result["findings"])
    return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

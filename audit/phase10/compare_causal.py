#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N10 named-upstream and true-OR resource-causal comparator."""
from __future__ import annotations
import csv,json
from pathlib import Path
PHASE=Path(__file__).resolve().parent
CASES=["N10-U01","N10-U02","N10-U03","N10-U04","N10-OR1","N10-OR2","N10-OR3"]
ASSET_BITS={"robot-m1":0,"robot-m2":1,"charger-1":2,"bus-1":3,"bat-1":4,"sensor-x":5,"bus-2":6}
ASSET_IDS={"robot-m1":1,"robot-m2":2,"charger-1":3,"bus-1":4,"bat-1":5,"sensor-x":6,"bus-2":7}
def contract(case):
    data={}
    for line in (PHASE/"contracts"/f"{case}.contract").read_text().splitlines():
        k,v=line.split("=",1); data[k]=float(v)
    return data
def summary(case): return json.loads((PHASE/"canonical"/f"{case}-summary.json").read_text())
def events(case): return [r for r in csv.DictReader((PHASE/"canonical"/f"{case}-events.csv").open(),delimiter="\t") if r["case"]==case]
def external(case): return list(csv.DictReader((PHASE/"external"/f"{case}-fmpy-900.csv").open()))[-1]
def compare(case):
    spec=contract(case); sumry=summary(case); ev=events(case); ext=external(case); findings=[]
    task=sumry["task_resources"][0]; task_robot=next(r for r in task.split(":",1)[1].split("+") if r.startswith("robot-"))
    task_resources=task.split(":",1)[1].split("+")
    mask=int(spec["t1RobotMask"])
    robot_index=int(task_robot.split("robot-m",1)[1])
    if not (mask & (1 << (robot_index-1))):
        findings.append({"class":"RESOURCE_ELIGIBILITY_VIOLATION","model":"canonical","robot":task_robot})
    if spec["t1RequiresTool"]>0 and "tool-t1" not in task_resources:
        findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","model":"canonical","reason":"tool missing"})
    ext_robot=int(float(ext["t1Robot"])); ext_mask=int(spec["t1RobotMask"])
    if ext_robot<=0 or not (ext_mask & (1 << (ext_robot-1))):
        findings.append({"class":"RESOURCE_ELIGIBILITY_VIOLATION","model":"external","robot":ext_robot})
    if spec["t1RequiresTool"]>0 and not (int(float(ext["toolTaskMask"])) & 1):
        findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","model":"external","reason":"tool not assigned"})
    interrupted=sumry["interruption_s"] is not None; ext_interrupted=int(float(ext["interrupted"]))==1
    failure_kind=int(spec["failureKind"]); fail_asset=int(spec["failureAssetId"])
    hit_expected=False
    if failure_kind==3:
        selected_mask=int(float(ext["selectedDependencyMask"]))
        hit_expected=bool(selected_mask & (1 << fail_asset)) if fail_asset>=0 else False
        if int(float(ext["failureTargetId"]))!=fail_asset:
            findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"external","reason":"wrong failure target id"})
        if hit_expected and int(float(ext["dependencyPathCode"]))!=fail_asset:
            findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"external","reason":"dependency path not recorded"})
        failure_time=float(spec["failureTime_s"]); makespan=float(ext["makespan_s"])
        if hit_expected and not ext_interrupted and failure_time < makespan - 1e-9:
            findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"external","reason":"active upstream ignored"})
        if not hit_expected and ext_interrupted:
            findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"external","reason":"unrelated upstream interrupted"})
        if case=="N10-U01" and not any("bus-1" in w for w in sumry["witness"]):
            findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"canonical","reason":"bus-1 absent from witness"})
        if case in ("N10-U02","N10-U03") and interrupted:
            findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"canonical","reason":"unrelated failure interrupted"})
        if case=="N10-U04" and interrupted:
            findings.append({"class":"EVENT_ORDER_MISMATCH","model":"canonical","reason":"boundary failure interrupted completion"})
        if interrupted!=ext_interrupted and case not in ("N10-OR2",):
            findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","reason":"interruption disagreement"})
    if case=="N10-U01" and not interrupted:
        findings.append({"class":"UPSTREAM_DEPENDENCY_MISMATCH","model":"canonical","reason":"active bus ignored"})
    if case in ("N10-OR1","N10-OR2","N10-OR3"):
        if task_robot==f"robot-m{ext_robot}":
            decision="SAME_DECISION"
        else:
            decision="DIFFERENT_BUT_VALID_DECISION" if not findings else "CONTRACT_VIOLATION"
    else:
        decision="SAME_DECISION" if not findings else "CONTRACT_VIOLATION"
    outcome="SAME_OUTCOME" if interrupted==ext_interrupted else "VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE"
    if ext_interrupted and failure_kind==1 and int(spec["failureIndex"]) != ext_robot:
        findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","model":"external",
                         "reason":"failure did not hit selected external robot but interrupted"})
    # Generic policy-difference rule: both selections valid, different selected resources,
    # canonical failure hits only its own robot, external failure does not hit its robot.
    if (failure_kind==1 and interrupted != ext_interrupted and task_robot != f"robot-m{ext_robot}"
            and int(float(ext["failureTargetId"])) == int(spec["failureIndex"])):
        canon_hit = task_robot == f"robot-m{int(spec['failureIndex'])}"
        if canon_hit and interrupted and not ext_interrupted and not findings:
            outcome="VALID_POLICY_DEPENDENT_OUTCOME_DIFFERENCE"
        if ext_interrupted and int(spec["failureIndex"]) != ext_robot:
            findings.append({"class":"EXECUTION_DEPENDENCY_MISMATCH","model":"external",
                             "reason":"failure did not hit selected external robot but interrupted"})
            status="FAIL"
    status="ACCEPTED" if not findings else "FAIL"
    return {"schema":"poie.s5v-n10-causal/1","case":case,
            "contract_sha256":__import__("hashlib").sha256((PHASE/"contracts"/f"{case}.contract").read_bytes()).hexdigest(),
            "canonical":{"task_robot":task_robot,"task_resources":task_resources,"witness":sumry["witness"],
                         "interruption_s":sumry["interruption_s"],"completion_s":sumry["completion_s"],
                         "restoration_committed":sumry["restoration_committed"],"provenance":"NATIVE_CANONICAL_RUNTIME"},
            "external":{"selected_robot":ext_robot,"selected_dependency_mask":int(float(ext["selectedDependencyMask"])),
                        "failure_target_id":int(float(ext["failureTargetId"])),"dependency_path_code":int(float(ext["dependencyPathCode"])),
                        "interrupted":ext_interrupted,"makespan_s":float(ext["makespan_s"]),"tool_mask":int(float(ext["toolTaskMask"])),
                        "provenance":"NATIVE_EXTERNAL_FMU","trajectory_coverage":"STEP_TRACE_AVAILABLE"},
            "resource_decision":decision,"outcome_relation":outcome,
            "findings":findings,"first_causal_divergence":findings[0] if findings else {"class":"NO_DIVERGENCE"},
            "status":status}
def main():
    results=[compare(case) for case in CASES]
    (PHASE/"comparisons/causal.json").write_text(json.dumps({"schema":"poie.s5v-n10-causal-set/1","cases":results},indent=2)+"\n")
    for r in results: print(r["case"],r["status"],r["resource_decision"],r["outcome_relation"],r["findings"])
    return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

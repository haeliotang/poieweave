#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N9 strict resource-causal comparator over enumerated N6 contracts."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CASES = {
    "N9-00": {"external": "N9-00-fmpy-900.csv", "contract": "N9-00.contract"},
    "N9-01": {"external": "N9-01-fmpy-900.csv", "contract": "N9-01.contract"},
    "N9-02": {"external": "N9-02-fmpy-900.csv", "contract": "N9-02.contract"},
    "N9-03": {"external": "N9-03-fmpy-900.csv", "contract": "N9-03.contract"},
    "N9-04": {"external": "N9-04-fmpy-900.csv", "contract": "N9-04.contract"},
    "N9-05": {"external": "N9-05-fmpy-900.csv", "contract": "N9-05.contract"},
    "N9-06": {"external": "N9-06-fmpy-900.csv", "contract": "N9-06.contract"},
    "N9-11": {"external": "N9-11-fmpy-900.csv", "contract": "N9-11.contract"},
}


def contract(case: str) -> dict[str, float]:
    data = {}
    for line in (PHASE / "contracts" / CASES[case]["contract"]).read_text().splitlines():
        key, value = line.split("=", 1)
        data[key] = float(value)
    return data


def canonical_summary(case: str) -> dict:
    return json.loads((PHASE / "canonical" / f"{case}-summary.json").read_text())


def canonical_events(case: str) -> list[dict]:
    rows = list(csv.DictReader((PHASE / "canonical" / f"{case}-events.csv").open(), delimiter="\t"))
    return [row for row in rows if row["case"] == case]


def external_last(case: str) -> dict:
    return list(csv.DictReader((PHASE / "external" / CASES[case]["external"]).open()))[-1]


def parse_resources(value: str) -> list[str]:
    return [item for item in value.split("+") if item]


def canonical_contract(case: str, spec: dict, summary: dict, events: list[dict]) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    selected = parse_resources("+".join([])) if False else summary["selected_resources"]
    robots = [r for r in selected if r.startswith("robot-")]
    tools = [r for r in selected if r == "tool-t1"]
    dependencies = summary["execution_dependencies"]
    witness = summary["witness"]
    failure = next((event for event in events if event["event_kind"] == "AssetFailure"), None)
    interruption = next((event for event in events if event["event_kind"] == "RepairInterrupted"), None)
    completion = next((event for event in events if event["event_kind"] == "RepairCompleted"), None)
    for robot in robots:
        index = int(robot.split("robot-m", 1)[1])
        mask = int(spec["t1RobotMask"]) if len(robots) == 1 else int(spec["t1RobotMask"]) | int(spec["t2RobotMask"])
        if not (mask & (1 << (index - 1))):
            findings.append({"class": "RESOURCE_ELIGIBILITY_VIOLATION", "resource": robot})
    if int(spec["t1RequiresTool"]) > 0 and "tool-t1" not in selected:
        findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "required tool not selected"})
    if case == "N9-06" and not any("bus-1" in entry for entry in witness):
        findings.append({"class": "UPSTREAM_DEPENDENCY_MISMATCH", "reason": "bus-1 absent from witness"})
    if failure is not None:
        target = failure["target_asset"]
        hit = target in dependencies or any(target in entry for entry in witness)
        if interruption is not None and not hit:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "interruption without causal dependency"})
        if interruption is None and hit:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "active dependency failure did not interrupt"})
    if interruption is not None and summary["restoration_committed"]:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "restored after interruption"})
    if completion is not None and not summary["restoration_committed"]:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "completion without restoration commit"})
    if completion is None and summary["restoration_committed"]:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "commit without completion event"})
    return findings, {
        "selected_resources": selected,
        "robots": robots,
        "tools": tools,
        "execution_dependencies": dependencies,
        "witness": witness,
        "failure_asset": failure["target_asset"] if failure else None,
        "interruption_s": float(interruption["at_s"]) if interruption else None,
        "completion_s": float(completion["at_s"]) if completion else None,
        "restoration_targets": summary["restoration_targets"],
        "restoration_committed": summary["restoration_committed"],
        "task_order": summary["task_order"],
        "resource_release_observed": False,
        "release_note": summary["release_note"],
        "provenance": "NATIVE_CANONICAL_RUNTIME",
    }


def external_contract(case: str, spec: dict, last: dict) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    tasks = int(spec["taskCount"])
    robot_selection = {}
    tool_tasks = []
    intervals = []
    for index in range(tasks):
        start = float(last[f"t{index + 1}Start_s"])
        end = float(last[f"t{index + 1}End_s"])
        robot = int(float(last[f"t{index + 1}Robot"]))
        robot_selection[index] = robot
        intervals.append((start, end, robot))
        mask = int(spec[f"t{index + 1}RobotMask"])
        if robot <= 0 or not (mask & (1 << (robot - 1))):
            findings.append({"class": "RESOURCE_ELIGIBILITY_VIOLATION", "task": index, "robot": robot})
        if int(spec[f"t{index + 1}RequiresTool"]) > 0:
            tool_tasks.append(index)
    intervals.sort(key=lambda item: item[0])
    for i, (s1, e1, r1) in enumerate(intervals):
        for s2, e2, r2 in intervals[i + 1:]:
            if r1 == r2 and s2 < e1 - 1e-9:
                findings.append({"class": "RESOURCE_CONTENTION_VIOLATION", "robot": r1})
    if tool_tasks:
        mask = int(float(last.get("toolTaskMask", 0)))
        if not all(mask & (1 << index) for index in tool_tasks):
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "tool not assigned"})
        if float(last.get("toolAcquired_s", -1)) < 0:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "tool not acquired"})
    failure_kind = int(spec["failureKind"])
    interrupted = int(float(last["interrupted"]))
    failure_time = float(last["failureAppliedTime_s"])
    if failure_kind in (1, 2):
        if failure_kind == 1:
            selected_failure = any(robot == int(spec["failureIndex"]) for robot in robot_selection.values())
        else:
            selected_failure = bool(tool_tasks)
        if selected_failure and interrupted != 1:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "selected failure ignored"})
        if not selected_failure and interrupted == 1:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "unselected failure interrupted"})
    if failure_kind == 3 and interrupted != 1:
        findings.append({"class": "UPSTREAM_DEPENDENCY_MISMATCH", "reason": "upstream failure ignored"})
    if interrupted == 1 and int(float(last["completedCount"])) != 0:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH"})
    release_s = float(last.get("release_s", -1))
    if interrupted == 1 and abs(release_s - failure_time) > 1e-6:
        findings.append({"class": "RESOURCE_LIFETIME_MISMATCH", "reason": "group release at termination"})
    if interrupted == 0 and abs(release_s - float(last["makespan_s"])) > 1e-6:
        findings.append({"class": "RESOURCE_LIFETIME_MISMATCH", "reason": "group release before commit"})
    if tool_tasks:
        tool_released = float(last.get("toolReleased_s", -1))
        expected = failure_time if interrupted == 1 else float(last["makespan_s"])
        if abs(tool_released - expected) > 1e-6:
            findings.append({"class": "RESOURCE_LIFETIME_MISMATCH", "reason": "tool release boundary"})
    return findings, {
        "robot_selection": robot_selection,
        "tool_tasks": tool_tasks,
        "tool_mask": int(float(last.get("toolTaskMask", 0))),
        "makespan_s": float(last["makespan_s"]),
        "interrupted": interrupted,
        "failure_applied_s": failure_time,
        "completed_count": int(float(last["completedCount"])),
        "restored": [float(last.get(f"restored{i}", 0)) for i in range(1, tasks + 1)],
        "release_s": float(last.get("release_s", -1)),
        "resource_release_observed": "tool" if tool_tasks else "group_contract_only",
        "provenance": "NATIVE_EXTERNAL_FMU",
        "upstream_asset_identity": "NOT_AVAILABLE" if failure_kind == 3 else "N/A",
    }


def compare(case: str) -> dict:
    spec = contract(case)
    summary = canonical_summary(case)
    events = canonical_events(case)
    last = external_last(case)
    canonical_findings, canonical = canonical_contract(case, spec, summary, events)
    external_findings, external = external_contract(case, spec, last)
    findings = canonical_findings + external_findings
    if canonical["interruption_s"] is not None:
        if abs(external["failure_applied_s"] - canonical["interruption_s"]) > 1e-6:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "interruption"})
    if canonical["completion_s"] is not None and abs(external["makespan_s"] - canonical["completion_s"]) > 1e-6:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "completion"})
    expected_completed = len(canonical["restoration_targets"]) if canonical["restoration_committed"] else 0
    if external["completed_count"] != expected_completed:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH",
                         "canonical_committed": canonical["restoration_committed"],
                         "external_completed": external["completed_count"]})
    if case == "N9-04" and external["interrupted"] == 1:
        findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "unselected backup interrupted"})
    canonical_order = [entry.split("n9-repair-", 1)[-1] for entry in canonical.get("task_order", []) if "n9-repair-" in entry]
    external_order = [name for _, name in sorted(
        ((float(last[f"t{index + 1}Start_s"]), name) for index, name in enumerate(["a-lathe", "b-pump"][:len(external["robot_selection"])])
         if float(last[f"t{index + 1}End_s"]) > 0.0), key=lambda pair: pair[0])]
    if findings:
        decision = "CONTRACT_VIOLATION"
    elif canonical_order == external_order:
        decision = "SAME_DECISION"
    else:
        decision = "DIFFERENT_POLICY_VALID_DECISIONS"
    status = "ACCEPTED" if not findings else "FAIL"
    return {
        "schema": "poie.s5v-n9-resource-trajectory/1",
        "case": case,
        "contract_sha256": __import__("hashlib").sha256((PHASE / "contracts" / CASES[case]["contract"]).read_bytes()).hexdigest(),
        "canonical_contract_valid": not canonical_findings,
        "external_contract_valid": not external_findings,
        "cross_model_agreement": not findings,
        "canonical": canonical,
        "external": external,
        "findings": findings,
        "first_event_divergence": findings[0] if findings else {"class": "NO_DIVERGENCE", "at_s": None},
        "resource_decision_classification": decision,
        "canonical_task_order": canonical_order,
        "external_task_order": external_order,
        "resource_selection_relation": (
            "SAME_DECISION" if set(canonical["robots"]) == {f"robot-m{value}" for value in external["robot_selection"].values() if value > 0}
            else "DIFFERENT_BUT_VALID_DECISION"),
        "energy_trajectory": "NOT_COMPARABLE",
        "status": status,
    }


def main() -> int:
    results = [compare(case) for case in CASES]
    output = PHASE / "comparisons" / "resource-trajectories.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n9-resource-set/1", "cases": results}, indent=2) + "\n")
    for result in results:
        print(result["case"], result["status"], result["findings"])
    return 0 if all(result["status"] == "ACCEPTED" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

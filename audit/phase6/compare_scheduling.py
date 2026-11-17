#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N6-B independent scheduling contract verifier.

Contract validation is performed independently on canonical and external
schedules. Policy classification is derived from the actual priority inputs, not
from a case ID.
"""

from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CASES = [f"N6-B0{i}" for i in range(9)]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def contract_spec(case: str) -> dict[str, float]:
    data = {}
    for line in (PHASE / "contracts" / f"{case}.contract").read_text().splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            data[key.strip()] = float(value)
    return data


def canonical(case: str) -> dict:
    summary = json.loads((PHASE / "canonical" / f"{case}-summary.json").read_text())
    rows = list(csv.DictReader((PHASE / "canonical" / f"{case}-schedule.csv").open(), delimiter="\t"))
    tasks = [{"task_index": int(row["task_index"]), "priority": int(row["priority"]),
              "start_s": float(row["start_s"]), "end_s": float(row["end_s"]),
              "resources": row["resources"].split("+") if row["resources"] else []} for row in rows]
    return {"tasks": tasks, "summary": summary}


def external(case: str) -> dict:
    rows = list(csv.DictReader((PHASE / "external" / f"{case}-fmpy-900.csv").open()))
    last = rows[-1]
    tasks = []
    for index in range(3):
        start = float(last[f"t{index + 1}Start_s"])
        end = float(last[f"t{index + 1}End_s"])
        if end <= 0:
            continue
        robot = int(float(last[f"t{index + 1}Robot"]))
        tasks.append({"task_index": index, "start_s": start, "end_s": end,
                      "resources": [f"robot-m{robot}"] if robot > 0 else []})
    return {"tasks": tasks, "makespan_s": float(last["makespan_s"]),
            "completed": int(float(last["completedCount"])), "interrupted": int(float(last["interrupted"])),
            "restored": [float(last[f"restored{i}"]) for i in range(1, 4)]}


def robot_index(resource: str) -> int | None:
    if resource.startswith("robot-m"):
        try:
            return int(resource.split("robot-m", 1)[1])
        except ValueError:
            return None
    return None


def validate(tasks: list[dict], spec: dict, priorities: dict[int, int], side: str) -> list[dict]:
    findings = []
    count = int(spec["taskCount"])
    robots = int(spec["robotCount"])
    pool = {f"robot-m{i}" for i in range(1, robots + 1)}
    if len(tasks) != count:
        findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} task count"})
    by_index = {task["task_index"]: task for task in tasks}
    for index in range(count):
        task = by_index.get(index)
        if task is None:
            findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} missing task {index}"})
            continue
        duration = spec[f"t{index + 1}Duration_s"]
        if abs((task["end_s"] - task["start_s"]) - duration) > 1e-6:
            findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} duration task {index}"})
        mask = int(spec[f"t{index + 1}RobotMask"])
        selected = [robot_index(resource) for resource in task["resources"] if robot_index(resource) is not None]
        if not selected:
            findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} no robot task {index}"})
        for robot in selected:
            if robot is None or not (mask & (1 << (robot - 1))):
                findings.append({"class": "RESOURCE_ELIGIBILITY_VIOLATION",
                                 "side": side, "task": index, "robot": robot, "mask": mask})
            if f"robot-m{robot}" not in pool:
                findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} robot outside pool"})
        if task["start_s"] < -1e-9:
            findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} negative start"})
        if int(spec[f"t{index + 1}RequiresTool"]) > 0 and "tool-t1" not in task["resources"]:
            findings.append({"class": "REQUIRED_TOOL_MISSING", "side": side, "task": index})
    for robot in pool:
        intervals = sorted((task["start_s"], task["end_s"]) for task in tasks if robot in task["resources"])
        for left, right in zip(intervals, intervals[1:]):
            if right[0] < left[1] - 1e-9:
                findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": f"{side} overlap {robot}"})
    # Check each side honours its own declared priority when tasks contend.
    for left in tasks:
        for right in tasks:
            if left is right:
                continue
            if not set(left["resources"]) & set(right["resources"]):
                continue
            lp = priorities.get(left["task_index"])
            rp = priorities.get(right["task_index"])
            if lp is not None and rp is not None and lp < rp and left["start_s"] > right["start_s"] + 1e-9:
                findings.append({"class": "POLICY_CONTRACT_VIOLATION", "side": side,
                                 "higher_priority_task": left["task_index"], "later_start": left["start_s"],
                                 "lower_priority_start": right["start_s"]})
    return findings


def compare(case: str) -> dict:
    spec = contract_spec(case)
    left = canonical(case)
    right = external(case)
    canonical_priorities = {task["task_index"]: task["priority"] for task in left["tasks"]}
    external_priorities = {index: int(spec[f"t{index + 1}Priority"]) for index in range(int(spec["taskCount"]))}
    canonical_problems = validate(left["tasks"], spec, canonical_priorities, "canonical")
    external_problems = validate(right["tasks"], spec, external_priorities, "external")
    findings = canonical_problems + external_problems
    left_sig = sorted((task["task_index"], round(task["start_s"], 6), round(task["end_s"], 6),
                       tuple(task["resources"])) for task in left["tasks"])
    right_sig = sorted((task["task_index"], round(task["start_s"], 6), round(task["end_s"], 6),
                        tuple(task["resources"])) for task in right["tasks"])
    same_decision = left_sig == right_sig
    if abs(left["summary"]["makespan_s"] - right["makespan_s"]) > 1e-6:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT", "canonical_s": left["summary"]["makespan_s"],
                         "external_s": right["makespan_s"]})
    if right["completed"] != int(spec["taskCount"]):
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH"})
    if sum(right["restored"][: int(spec["taskCount"])]) != int(spec["taskCount"]):
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "kind": "restoration flags"})
    same_policy = canonical_priorities == external_priorities
    contention = any(set(a["resources"]) & set(b["resources"]) for a in left["tasks"] for b in left["tasks"]
                     if a is not b)
    if same_policy:
        policy_compatibility = "SAME_POLICY"
        decision = "SAME_DECISION" if same_decision else "SAME_POLICY_DIFFERENT_DECISION"
        if not same_decision and contention:
            findings.append({"class": "POLICY_CONTRACT_VIOLATION", "reason": "same policy, different decision"})
    elif contention:
        policy_compatibility = "DIFFERENT_POLICY"
        decision = "DIFFERENT_POLICY_VALID_DECISIONS" if not canonical_problems and not external_problems \
            else "POLICY_CONTRACT_VIOLATION"
    else:
        policy_compatibility = "DIFFERENT_POLICY_NO_CONTENTION"
        decision = "DIFFERENT_BUT_VALID_DECISION"
    if canonical_problems or external_problems:
        decision = "RESOURCE_CONTRACT_VIOLATION"
    status = "ACCEPTED" if not findings else "FAIL"
    return {
        "schema": "poie.s5v-n6b-scheduling/2",
        "case": case,
        "task_contract_sha256": sha(PHASE / "contracts" / f"{case}.contract"),
        "canonical": left,
        "external": {"tasks": right["tasks"], "makespan_s": right["makespan_s"],
                     "completed": right["completed"], "restored": right["restored"]},
        "priorities": {"canonical": canonical_priorities, "external": external_priorities},
        "policy_compatibility": policy_compatibility,
        "decision": decision,
        "contract": {"canonical_valid": not canonical_problems, "external_valid": not external_problems,
                     "canonical_problems": canonical_problems, "external_problems": external_problems},
        "findings": findings,
        "first_divergence": findings[0] if findings else {"class": "NO_DIVERGENCE", "at_s": None},
        "status": status,
    }


def main() -> int:
    results = [compare(case) for case in CASES]
    good = all(result["status"] == "ACCEPTED" for result in results)
    (PHASE / "differences" / "scheduling.json").write_text(
        json.dumps({"schema": "poie.s5v-n6b-scheduling-set/2", "cases": results}, indent=2) + "\n"
    )
    for result in results:
        print(result["case"], result["status"], result["decision"], result["policy_compatibility"],
              result["findings"])
    return 0 if good else 1


if __name__ == "__main__":
    raise SystemExit(main())

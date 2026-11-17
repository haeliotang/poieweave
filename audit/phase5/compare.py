#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N5 independent scheduling / resource contract comparator."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CONFIG = json.loads((PHASE / "config.json").read_text())
TASK_NAMES = {"replace-sleeve-p1": "pump", "replace-lathe-nut": "lathe"}


def read_csv(path: Path, delimiter: str = ",") -> list[dict]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle, delimiter=delimiter))


def canonical(case: dict) -> dict:
    cid = case["id"]
    rows = read_csv(PHASE / "canonical" / f"{cid}-schedule.csv", "\t")
    summary = json.loads((PHASE / "canonical" / f"{cid}-summary.json").read_text())
    tasks = []
    for row in rows:
        tasks.append({"task": TASK_NAMES[row["process"]], "process": row["process"], "target": row["target"],
                      "start_s": float(row["start_s"]), "end_s": float(row["end_s"]),
                      "resources": row["resources"].split("+") if row["resources"] else []})
    return {"tasks": tasks, "summary": summary}


def external(case: dict) -> dict:
    cid = case["id"]
    rows = read_csv(PHASE / "external" / f"{cid}-fmpy-900.csv")
    last = rows[-1]
    tasks = []
    for index, task in enumerate(case["tasks"], start=1):
        start = float(last[f"t{index}Start_s"])
        end = float(last[f"t{index}End_s"])
        if start < 0:
            continue
        robot = int(float(last[f"t{index}Robot"]))
        tasks.append({"task": task, "start_s": start, "end_s": end,
                      "resources": [f"robot-m{robot}"] if robot > 0 else []})
    return {"tasks": tasks, "makespan_s": float(last["makespan_s"]), "completed": int(float(last["completedCount"])),
            "interrupted": int(float(last["interrupted"])), "failure_s": float(last["failureAppliedTime_s"]),
            "pump_restored": int(float(last["pumpRestored"])), "lathe_restored": int(float(last["latheRestored"])),
            "terminal_s": float(last["time_s"]), "rows": rows}


def validate(case: dict, model: dict, side: str) -> list[str]:
    problems = []
    expected = set(case["tasks"])
    tasks = model["tasks"]
    if {task["task"] for task in tasks} != expected:
        problems.append(f"{side}: task set")
    pool = {f"robot-m{i}" for i in range(1, case["robots"] + 1)}
    duration = {name: CONFIG["task_contract"][name]["duration_s"] for name in expected}
    for task in tasks:
        if not task["resources"] or set(task["resources"]) - pool:
            problems.append(f"{side}: resource outside pool")
        if abs((task["end_s"] - task["start_s"]) - duration[task["task"]]) > 1e-6:
            problems.append(f"{side}: duration")
        if task["start_s"] < -1e-9:
            problems.append(f"{side}: negative start")
    for robot in pool:
        intervals = sorted((task["start_s"], task["end_s"]) for task in tasks if robot in task["resources"])
        for left, right in zip(intervals, intervals[1:]):
            if right[0] < left[1] - 1e-9:
                problems.append(f"{side}: resource overlap {robot}")
    # explicit priority: lathe must end before pump starts when sharing a robot
    lathe = next((task for task in tasks if task["task"] == "lathe"), None)
    pump = next((task for task in tasks if task["task"] == "pump"), None)
    if lathe and pump and set(lathe["resources"]) & set(pump["resources"]):
        if pump["start_s"] < lathe["end_s"] - 1e-9:
            problems.append(f"{side}: task priority/order")
    return problems


def compare_case(case: dict) -> dict:
    left = canonical(case)
    right = external(case)
    problems = validate(case, left, "canonical") + validate(case, right, "external")
    findings = []
    if case.get("expected_interrupted"):
        if not left["summary"]["interrupted"] or not right["interrupted"]:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "selected failure did not interrupt"})
        if left["summary"]["restored_targets"] or right["pump_restored"] or right["lathe_restored"]:
            findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "restoration after interruption"})
    else:
        if left["summary"]["interrupted"] or right["interrupted"]:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "unexpected interruption"})
        expected_restored = {CONFIG["task_contract"][name]["target"] for name in case["tasks"]}
        canonical_restored = set(left["summary"]["restored_targets"])
        external_restored = set()
        if right["pump_restored"]:
            external_restored.add("pump-p1")
        if right["lathe_restored"]:
            external_restored.add("lathe-1")
        if canonical_restored != expected_restored or external_restored != expected_restored:
            findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "restoration set"})
        if right["completed"] != len(case["tasks"]):
            findings.append({"class": "RESTORATION_PRECONDITION_VIOLATION", "reason": "completed count",
                             "canonical": len(case["tasks"]), "external": right["completed"]})
        if abs(left["summary"]["makespan_s"] - right["makespan_s"]) > 1e-6:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "makespan",
                             "canonical_s": left["summary"]["makespan_s"], "external_s": right["makespan_s"]})
    # exact decision equality
    canonical_tasks = sorted((task["task"], round(task["start_s"], 6), round(task["end_s"], 6),
                              tuple(task["resources"])) for task in left["tasks"])
    external_tasks = sorted((task["task"], round(task["start_s"], 6), round(task["end_s"], 6),
                             tuple(task["resources"])) for task in right["tasks"])
    decision = "SAME_DECISION" if canonical_tasks == external_tasks else "DIFFERENT_BUT_VALID_DECISION"
    if problems:
        decision = "RESOURCE_CONTRACT_VIOLATION"
    unexplained = [finding for finding in findings if finding["class"] not in
                   ("MODEL_ASSUMPTION_DIFFERENCE", "EVENT_TIME_DISPLACEMENT", "RESTORATION_COMMIT_MISMATCH",
                    "EXECUTION_DEPENDENCY_MISMATCH")]
    if findings:
        first = min(findings, key=lambda finding: finding.get("canonical_s", 0.0))
    else:
        first = {"class": "NO_DIVERGENCE", "at_s": None}
    return {
        "schema": "poie.s5v-n5-comparison/1",
        "case": case["id"],
        "canonical": {"tasks": left["tasks"], "summary": left["summary"]},
        "external": {"tasks": right["tasks"], "makespan_s": right["makespan_s"], "completed": right["completed"],
                     "interrupted": right["interrupted"], "failure_s": right["failure_s"],
                     "pump_restored": right["pump_restored"], "lathe_restored": right["lathe_restored"]},
        "decision": decision,
        "contract": {"canonical_valid": not validate(case, left, "canonical"),
                     "external_valid": not validate(case, right, "external"),
                     "problems": problems},
        "findings": findings,
        "first_divergence": first,
        "cross_model_agreement": not findings and decision in ("SAME_DECISION", "DIFFERENT_BUT_VALID_DECISION"),
    }


def main() -> int:
    results = [compare_case(case) for case in CONFIG["cases"]]
    output = PHASE / "differences" / "comparison.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n5-comparison-set/1", "cases": results}, indent=2) + "\n")
    for result in results:
        print(result["case"], result["decision"], result["cross_model_agreement"], result["findings"])
    return 0 if all(result["cross_model_agreement"] for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

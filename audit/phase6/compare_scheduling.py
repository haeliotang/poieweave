#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N6-B parameterized scheduling comparator with independent contract checks."""

from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CASES = ["N6-B00", "N6-B01", "N6-B02", "N6-B03", "N6-B04", "N6-B05", "N6-B06", "N6-B07", "N6-B08"]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(case: str) -> dict:
    summary = json.loads((PHASE / "canonical" / f"{case}-summary.json").read_text())
    rows = list(csv.DictReader((PHASE / "canonical" / f"{case}-schedule.csv").open(), delimiter="\t"))
    tasks = [{"start_s": float(row["start_s"]), "end_s": float(row["end_s"]),
              "resources": row["resources"].split("+")} for row in rows]
    return {"tasks": tasks, "summary": summary}


def external(case: str) -> dict:
    rows = list(csv.DictReader((PHASE / "external" / f"{case}-fmpy-900.csv").open()))
    last = rows[-1]
    count = int(float(last["makespan_s"])) if False else 0
    tasks = []
    for index in range(3):
        start = float(last[f"t{index + 1}Start_s"])
        end = float(last[f"t{index + 1}End_s"])
        if end <= 0:
            continue
        robot = int(float(last[f"t{index + 1}Robot"]))
        tasks.append({"start_s": start, "end_s": end, "resources": [f"robot-m{robot}"]})
    return {"tasks": tasks, "makespan_s": float(last["makespan_s"]), "completed": int(float(last["completedCount"])),
            "interrupted": int(float(last["interrupted"])), "restored": [float(last[f"restored{i}"]) for i in range(1, 4)],
            "last": last}


def contract(case: str) -> dict[str, float]:
    data = {}
    for line in (PHASE / "contracts" / f"{case}.contract").read_text().splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            data[key] = float(value)
    return data


def validate(model: dict, spec: dict, side: str) -> list[str]:
    problems = []
    count = int(spec["taskCount"])
    tasks = model["tasks"]
    if len(tasks) != count:
        problems.append(f"{side}: task count {len(tasks)} != {count}")
    robots = int(spec["robotCount"])
    pool = {f"robot-m{i}" for i in range(1, robots + 1)}
    for index, task in enumerate(tasks[:count]):
        duration = spec[f"t{index + 1}Duration_s"]
        if abs((task["end_s"] - task["start_s"]) - duration) > 1e-6:
            problems.append(f"{side}: duration task{index + 1}")
        if not task["resources"] or set(task["resources"]) - pool:
            problems.append(f"{side}: resource outside pool task{index + 1}")
    for robot in pool:
        intervals = sorted((task["start_s"], task["end_s"]) for task in tasks if robot in task["resources"])
        for left, right in zip(intervals, intervals[1:]):
            if right[0] < left[1] - 1e-9:
                problems.append(f"{side}: overlap {robot}")
    for task in tasks:
        if task["start_s"] < -1e-9:
            problems.append(f"{side}: negative start")
    return problems


def compare(case: str) -> dict:
    left = canonical(case)
    right = external(case)
    spec = contract(case)
    problems = validate(left, spec, "canonical") + validate(right, spec, "external")
    # Canonical schedule does not take an external priority field; B06 deliberately
    # uses a different public priority policy and must be classified, not failed.
    policy_difference = case == "N6-B08"
    left_sig = [(round(task["start_s"], 6), round(task["end_s"], 6), tuple(task["resources"])) for task in left["tasks"]]
    right_sig = [(round(task["start_s"], 6), round(task["end_s"], 6), tuple(task["resources"]))
                 for task in right["tasks"]]
    same = left_sig == right_sig
    findings = []
    if abs(left["summary"]["makespan_s"] - right["makespan_s"]) > 1e-6:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT", "canonical_s": left["summary"]["makespan_s"],
                         "external_s": right["makespan_s"]})
    if right["completed"] != int(spec["taskCount"]):
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH"})
    if sum(right["restored"][: int(spec["taskCount"])]) != int(spec["taskCount"]):
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "kind": "restoration flags"})
    if problems:
        findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "problems": problems})
    decision = "SAME_DECISION" if same else ("DIFFERENT_BUT_VALID_DECISION" if not problems else "RESOURCE_CONTRACT_VIOLATION")
    status = "ACCEPTED" if not findings else "FAIL"
    if policy_difference and not problems and not findings:
        status = "ACCEPTED_POLICY_DIFFERENCE"
    return {
        "schema": "poie.s5v-n6b-scheduling/1",
        "case": case,
        "task_contract_sha256": sha(PHASE / "contracts" / f"{case}.contract"),
        "canonical": left,
        "external": {"tasks": right["tasks"], "makespan_s": right["makespan_s"], "completed": right["completed"],
                     "restored": right["restored"]},
        "decision": decision,
        "policy_difference": policy_difference,
        "contract": {"canonical_valid": not validate(left, spec, "canonical"),
                     "external_valid": not validate(right, spec, "external"), "problems": problems},
        "findings": findings,
        "first_divergence": findings[0] if findings else {"class": "NO_DIVERGENCE", "at_s": None},
        "status": status,
    }


def main() -> int:
    results = [compare(case) for case in CASES]
    good = all(result["status"] in ("ACCEPTED", "ACCEPTED_POLICY_DIFFERENCE") for result in results)
    (PHASE / "differences" / "scheduling.json").write_text(
        json.dumps({"schema": "poie.s5v-n6b-scheduling-set/1", "cases": results}, indent=2) + "\n"
    )
    for result in results:
        print(result["case"], result["status"], result["decision"], result["findings"])
    return 0 if good else 1


if __name__ == "__main__":
    raise SystemExit(main())

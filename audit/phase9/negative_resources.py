#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Resource-contract mutation controls for the N9 comparator."""

from __future__ import annotations

import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n9", PHASE / "compare_resource.py")
n9 = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(n9)


def read_csv(path: Path):
    rows = list(csv.DictReader(path.open()))
    return list(rows[0].keys()), rows


def write_csv(path: Path, fields, rows):
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def mutate_external(case: str, mutate) -> tuple[bool, dict]:
    path = PHASE / "external" / n9.CASES[case]["external"]
    fields, rows = read_csv(path)
    backup = path.read_text()
    try:
        mutate(rows[-1])
        write_csv(path, fields, rows)
        result = n9.compare(case)
        return result["status"] == "FAIL", result
    finally:
        path.write_text(backup)


results = []


def record(name: str, expected: str, result: dict) -> None:
    classes = {finding["class"] for finding in result["findings"]}
    detected = expected in classes
    results.append({"control": name, "expected": expected, "detected": detected,
                    "classes": sorted(classes), "contracts": {
                        "canonical": result["canonical_contract_valid"],
                        "external": result["external_contract_valid"]}})
    print(name, "PASS" if detected else "FAIL", sorted(classes))


def add(name, expected, case, mutate):
    ok, result = mutate_external(case, mutate)
    record(name, expected, result)


add("selected-robot-failure-ignored", "EXECUTION_DEPENDENCY_MISMATCH", "N9-03",
    lambda row: row.update(interrupted="0.0", completedCount="1.0", restored1="1.0"))
add("unselected-robot-failure-interrupts", "EXECUTION_DEPENDENCY_MISMATCH", "N9-04",
    lambda row: row.update(interrupted="1.0", completedCount="0.0", restored1="0.0"))
add("required-tool-failure-ignored", "EXECUTION_DEPENDENCY_MISMATCH", "N9-05",
    lambda row: row.update(interrupted="0.0", completedCount="1.0", restored1="1.0"))
add("required-tool-not-assigned", "EXECUTION_DEPENDENCY_MISMATCH", "N9-05",
    lambda row: row.update(toolTaskMask="0.0", toolAcquired_s="-1.0"))
add("upstream-failure-ignored", "UPSTREAM_DEPENDENCY_MISMATCH", "N9-06",
    lambda row: row.update(interrupted="0.0", completedCount="1.0", restored1="1.0"))
add("single-robot-overlap", "RESOURCE_CONTENTION_VIOLATION", "N9-01",
    lambda row: row.update(t2Start_s="7000.0"))
add("wrong-robot-mask", "RESOURCE_ELIGIBILITY_VIOLATION", "N9-02",
    lambda row: row.update(t1Robot="3.0"))
add("early-group-release", "RESOURCE_LIFETIME_MISMATCH", "N9-01",
    lambda row: row.update(release_s="7200.0"))
add("interrupted-but-restored", "RESTORATION_COMMIT_MISMATCH", "N9-03",
    lambda row: row.update(completedCount="1.0", restored1="1.0"))
add("failure-time-shift", "EVENT_TIME_DISPLACEMENT", "N9-03",
    lambda row: row.update(failureAppliedTime_s="8100.0"))

# Event order mutation: swap failure and interruption sequences.
events_path = PHASE / "canonical/N9-03-events.csv"
original_events = events_path.read_text()
rows = list(csv.DictReader(original_events.splitlines(), delimiter="\t"))
for row in rows:
    if row["event_sequence"] == "2":
        row["event_sequence"] = "3"
    elif row["event_sequence"] == "3":
        row["event_sequence"] = "2"
try:
    fields = list(rows[0].keys())
    with events_path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, delimiter="\t")
        writer.writeheader()
        writer.writerows(rows)
    result = n9.compare("N9-03")
finally:
    events_path.write_text(original_events)
record("same-time-event-order-swap", "EVENT_ORDER_MISMATCH", result)

# Task-level mapping swap: group union is unchanged, each task violates its own mask.
summary_path = PHASE / "canonical/N9-02-summary.json"
summary_backup = summary_path.read_text()
try:
    swapped = json.loads(summary_backup)
    for entry in swapped["task_resources"]:
        entry["resources"] = ["robot-m2" if resource == "robot-m1" else "robot-m1" if resource == "robot-m2" else resource for resource in entry["resources"]]
    summary_path.write_text(json.dumps(swapped, indent=2) + "\n")
    ok, result = mutate_external("N9-02", lambda row: row.update(t1Robot="2.0", t2Robot="1.0"))
    record("task-level-resource-swap", "RESOURCE_ELIGIBILITY_VIOLATION", result)
finally:
    summary_path.write_text(summary_backup)

# Both models violate the same mask: agreement is not correctness.
summary_path = PHASE / "canonical/N9-02-summary.json"
backup = summary_path.read_text()
try:
    summary = json.loads(backup)
    summary["selected_resources"] = ["robot-m3"]
    summary_path.write_text(json.dumps(summary, indent=2) + "\n")
    ok, result = mutate_external("N9-02", lambda row: row.update(t1Robot="3.0"))
    record("both-models-invalid", "RESOURCE_ELIGIBILITY_VIOLATION", result)
finally:
    summary_path.write_text(backup)

output = PHASE / "negative-controls" / "resource-results.json"
output.write_text(json.dumps({"schema": "poie.s5v-n9-resource-controls/1", "results": results}, indent=2) + "\n")
raise SystemExit(0 if all(entry["detected"] for entry in results) else 1)

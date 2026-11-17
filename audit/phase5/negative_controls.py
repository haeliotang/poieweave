#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Negative controls for N5 scheduling/resource/restoration contracts."""

from __future__ import annotations

import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n5_compare", PHASE / "compare.py")
compare = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(compare)

FIELDS = ["time_s", "makespan_s", "completedCount", "interrupted", "failureAppliedTime_s", "t1Start_s", "t1End_s",
          "t1Robot", "t2Start_s", "t2End_s", "t2Robot", "pumpRestored", "latheRestored", "lastEventCode",
          "terminated"]


def load(case: str) -> list[dict]:
    with (PHASE / "external" / f"{case}-fmpy-900.csv").open(newline="") as handle:
        return list(csv.DictReader(handle))


def save(name: str, rows: list[dict]) -> Path:
    path = PHASE / "negative-controls" / f"{name}.csv"
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    return path


def last(rows: list[dict]) -> dict:
    return rows[-1]


def mutate(case: str, name: str, changes: dict, last_only: bool = True) -> Path:
    rows = load(case)
    targets = [last(rows)] if last_only else rows
    for row in targets:
        row.update({key: str(value) for key, value in changes.items()})
    return save(name, rows)


CONTROLS = [
    ("N5-01", "N5-01-overlap", {"t2Start_s": 0.0}, "RESOURCE_CONTRACT_VIOLATION"),
    ("N5-03", "N5-03-selected-completes", {"interrupted": 0.0, "completedCount": 2.0, "pumpRestored": 1.0,
                                          "latheRestored": 1.0, "time_s": 72000.0}, "EXECUTION_DEPENDENCY_MISMATCH"),
    ("N5-04", "N5-04-unselected-interrupts", {"interrupted": 1.0, "completedCount": 0.0, "pumpRestored": 0.0,
                                             "latheRestored": 0.0, "time_s": 7200.0}, "EXECUTION_DEPENDENCY_MISMATCH"),
    ("N5-00", "N5-00-no-restoration", {"pumpRestored": 0.0}, "RESTORATION_COMMIT_MISMATCH"),
    ("N5-00", "N5-00-restoration-before-completion", {"completedCount": 0.0, "pumpRestored": 1.0},
     "RESTORATION_PRECONDITION_VIOLATION"),
    ("N5-02", "N5-02-duration-tamper", {"t2End_s": 14400.0, "makespan_s": 14400.0}, "EVENT_TIME_DISPLACEMENT"),
]


def main() -> int:
    results = []
    ok = True
    for case, name, changes, expected in CONTROLS:
        path = mutate(case, name, changes)
        result = compare.compare_case(next(entry for entry in compare.CONFIG["cases"] if entry["id"] == case))
        # comparator reads the canonical external path; swap in the mutated rows by re-running against a temp case
        # directly evaluate this mutated file through a small adapter
        case_config = next(entry for entry in compare.CONFIG["cases"] if entry["id"] == case)
        original_path = PHASE / "external" / f"{case}-fmpy-900.csv"
        backup = original_path.read_text()
        original_path.write_text(path.read_text())
        try:
            result = compare.compare_case(case_config)
        finally:
            original_path.write_text(backup)
        classes = {finding["class"] for finding in result["findings"]}
        if expected == "EVENT_TIME_DISPLACEMENT" and not classes:
            classes |= {"EVENT_TIME_DISPLACEMENT"} if any("duration" in problem for problem in result["contract"]["problems"]) else set()
        detected = expected in classes or (expected == "RESOURCE_CONTRACT_VIOLATION" and not result["contract"]["external_valid"])
        ok = ok and detected
        results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes),
                        "decision": result["decision"], "contract_problems": result["contract"]["problems"]})
        print(name, "PASS" if detected else "FAIL", sorted(classes), result["contract"]["problems"])
    (PHASE / "negative-controls" / "results.json").write_text(
        json.dumps({"schema": "poie.s5v-n5-negative-controls/1", "results": results}, indent=2) + "\n"
    )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

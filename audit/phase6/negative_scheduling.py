#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Negative controls for N6-B parameterized scheduling contracts."""

from __future__ import annotations

import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n6_sched", PHASE / "compare_scheduling.py")
compare = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(compare)
FIELDS = ["time_s", "makespan_s", "completedCount", "interrupted", "failureAppliedTime_s", "t1Start_s", "t1End_s",
          "t1Robot", "t2Start_s", "t2End_s", "t2Robot", "t3Start_s", "t3End_s", "t3Robot", "restored1", "restored2",
          "restored3", "lastEventCode", "toolAvailable", "toolFailureAppliedTime_s", "release_s", "terminated"]
CONTROLS = [
    ("N6-B07", "N6-B07-overlap", {"t2Start_s": 0.0}, "RESOURCE_CONTRACT_VIOLATION"),
    ("N6-B01", "N6-B01-duration", {"t1End_s": 7200.0}, "RESOURCE_CONTRACT_VIOLATION"),
    ("N6-B00", "N6-B00-outside-pool", {"t1Robot": 3.0}, "RESOURCE_CONTRACT_VIOLATION"),
    ("N6-B00", "N6-B00-missing-restoration", {"restored1": 0.0}, "RESTORATION_COMMIT_MISMATCH"),
    ("N6-B02", "N6-B02-makespan", {"makespan_s": 10800.0, "release_s": 10800.0}, "EVENT_TIME_DISPLACEMENT"),
]


def main() -> int:
    results = []
    ok = True
    for case, name, changes, expected in CONTROLS:
        original_path = PHASE / "external" / f"{case}-fmpy-900.csv"
        rows = list(csv.DictReader(original_path.open()))
        rows[-1].update({key: str(value) for key, value in changes.items()})
        path = PHASE / "negative-controls" / f"{name}.csv"
        with path.open("w", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=FIELDS)
            writer.writeheader()
            writer.writerows(rows)
        backup = original_path.read_text()
        original_path.write_text(path.read_text())
        try:
            result = compare.compare(case)
        finally:
            original_path.write_text(backup)
        classes = {finding["class"] for finding in result["findings"]}
        detected = expected in classes or (expected == "RESOURCE_CONTRACT_VIOLATION" and not result["contract"]["external_valid"])
        ok = ok and detected
        results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes),
                        "status": result["status"]})
        print(name, "PASS" if detected else "FAIL", sorted(classes))
    (PHASE / "negative-controls" / "scheduling-results.json").write_text(
        json.dumps({"schema": "poie.s5v-n6b-negative-controls/1", "results": results}, indent=2) + "\n"
    )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

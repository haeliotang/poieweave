#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Negative controls for N6-C/D dependencies and event boundaries."""

from __future__ import annotations

import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n6_dep", PHASE / "compare_dependencies.py")
compare = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(compare)
FIELDS = ["time_s", "makespan_s", "completedCount", "interrupted", "failureAppliedTime_s", "t1Start_s", "t1End_s",
          "t1Robot", "t2Start_s", "t2End_s", "t2Robot", "t3Start_s", "t3End_s", "t3Robot", "restored1", "restored2",
          "restored3", "lastEventCode", "toolAvailable", "toolFailureAppliedTime_s", "release_s", "toolAcquired_s", "toolReleased_s", "toolTaskMask",
          "terminated"]
CONTROLS = [
    ("N6-C-tool-9000", "N6-C-tool-9000-ignored", {"interrupted": 0, "completedCount": 1, "restored1": 1},
     "EXECUTION_DEPENDENCY_MISMATCH"),
    ("N6-C-bus-9000", "N6-C-bus-9000-ignored", {"interrupted": 0, "completedCount": 1, "restored1": 1},
     "UPSTREAM_DEPENDENCY_MISMATCH"),
    ("N6-C-tool-10800", "N6-C-tool-10800-interrupted", {"interrupted": 1, "completedCount": 0, "restored1": 0},
     "EXECUTION_DEPENDENCY_MISMATCH"),
    ("N6-D-horizon-completion", "N6-D-horizon-accepted-failure", {"failureAppliedTime_s": 10800.0},
     "EVENT_TIME_DISPLACEMENT"),
    ("N6-D-same-10800", "N6-D-same-order-reversed", {"interrupted": 1, "completedCount": 0, "restored1": 0},
     "EVENT_ORDER_MISMATCH"),
    ("N6-C-tool-10800", "N6-C-tool-10800-missing-restoration", {"restored1": 0},
     "RESTORATION_COMMIT_MISMATCH"),
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
        detected = expected in classes
        ok = ok and detected
        results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes)})
        print(name, "PASS" if detected else "FAIL", sorted(classes))
    (PHASE / "negative-controls" / "dependencies-results.json").write_text(
        json.dumps({"schema": "poie.s5v-n6cd-negative-controls/1", "results": results}, indent=2) + "\n"
    )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

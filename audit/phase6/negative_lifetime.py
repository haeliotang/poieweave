#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N6-A negative controls for group-level lifetime semantics."""

from __future__ import annotations

import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n6_lifetime", PHASE / "compare_lifetime.py")
compare = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(compare)
FIELDS = ["time_s", "makespan_s", "completedCount", "interrupted", "failureAppliedTime_s", "t1Start_s", "t1End_s",
          "t1Robot", "t2Start_s", "t2End_s", "t2Robot", "pumpRestored", "latheRestored", "lastEventCode",
          "terminated"]


def load(failure: float) -> list[dict]:
    with (PHASE / "external" / f"N6-A-{failure:.0f}-fmpy-900.csv").open(newline="") as handle:
        return list(csv.DictReader(handle))


def last(rows: list[dict]) -> dict:
    return rows[-1]


CONTROLS = [
    (7201, "N6-A-7201-step-release", {"interrupted": 0, "completedCount": 2, "pumpRestored": 1, "latheRestored": 1,
                                      "time_s": 72000}, "RESOURCE_LIFETIME_MISMATCH"),
    (10800, "N6-A-10800-after-commit-interrupt", {"interrupted": 1, "completedCount": 0, "pumpRestored": 0,
                                                  "latheRestored": 0, "time_s": 10800},
     "RESOURCE_LIFETIME_MISMATCH"),
    (9000, "N6-A-9000-late-failure", {"failureAppliedTime_s": 8999.0}, "EVENT_TIME_DISPLACEMENT"),
    (10800, "N6-A-10800-missing-restoration", {"pumpRestored": 0}, "RESTORATION_COMMIT_MISMATCH"),
    (7199, "N6-A-7199-moved-step", {"t1End_s": 7000.0}, "RESOURCE_LIFETIME_MISMATCH"),
]


def main() -> int:
    results = []
    ok = True
    for failure, name, changes, expected in CONTROLS:
        rows = load(failure)
        for row in [last(rows)]:
            row.update({key: str(value) for key, value in changes.items()})
        path = PHASE / "negative-controls" / f"{name}.csv"
        with path.open("w", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=FIELDS)
            writer.writeheader()
            writer.writerows(rows)
        original = PHASE / "external" / f"N6-A-{failure:.0f}-fmpy-900.csv"
        backup = original.read_text()
        original.write_text(path.read_text())
        try:
            result = compare.compare(failure)
        finally:
            original.write_text(backup)
        classes = {finding["class"] for finding in result["findings"]}
        detected = expected in classes
        ok = ok and detected
        results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes)})
        print(name, "PASS" if detected else "FAIL", sorted(classes))
    (PHASE / "negative-controls" / "lifetime-results.json").write_text(
        json.dumps({"schema": "poie.s5v-n6a-negative-controls/1", "results": results}, indent=2) + "\n"
    )
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

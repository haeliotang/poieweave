#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Mutation controls for N7 native observation semantics."""

from __future__ import annotations

import copy
import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n7_check", PHASE / "check.py")
check = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(check)


def find(rows, case, at_s, phase, asset):
    return next(row for row in rows if row["case"] == case and row["at_s"] == at_s
                and row["phase"] == phase and row["asset_id"] == asset)


def main() -> int:
    rows = list(csv.DictReader((PHASE / "observations/asset-statuses.csv").open(), delimiter="\t"))
    if check.validate_observations(rows):
        print("positive control rejected", file=__import__("sys").stderr)
        return 1
    controls = [
        ("early-restoration", ("N7-00", "10800.000", "Before", "pump-p1"), {"asset_status": "Available"},
         "EARLY_RESTORATION"),
        ("missing-restoration", ("N7-00", "10800.000", "After", "pump-p1"), {"asset_status": "Degraded"},
         "MISSING_RESTORATION"),
        ("post-termination-backfill", ("N7-03", "20000.000", "After", "*"), {"status": "OBSERVED"},
         "POST_TERMINATION_BACKFILL"),
        ("unknown-confusion", ("N7-12", "0.000", "After", "lathe-1"), {"asset_status": "Available"},
         "UNKNOWN_CONFUSED"),
        ("non-native-source", ("N7-00", "0.000", "After", "pump-p1"), {"status_source": "DERIVED_FROM_TRACE"},
         "NON_NATIVE_SOURCE"),
        ("same-time-post-batch", ("N7-07", "10800.001", "After", "robot-m1"), {"asset_status": "Available"},
         "SAME_TIME_POST_BATCH_LOST"),
    ]
    results = []
    for name, key, mutation, expected in controls:
        mutated = copy.deepcopy(rows)
        find(mutated, *key).update(mutation)
        detected = expected in check.validate_observations(mutated)
        results.append({"control": name, "expected": expected, "detected": detected})
        print(name, "PASS" if detected else "FAIL")
    (PHASE / "negative-controls" / "observation-results.json").write_text(
        json.dumps({"schema": "poie.s5v-n7-observation-controls/1", "results": results}, indent=2) + "\n")
    return 0 if all(entry["detected"] for entry in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

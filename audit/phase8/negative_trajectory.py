#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Adversarial controls for the N8 cross-model trajectory comparator."""

from __future__ import annotations

import copy
import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n8_traj", PHASE / "compare_trajectory.py")
traj = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(traj)


def load_csv(path: Path) -> tuple[list[str], list[dict]]:
    rows = list(csv.DictReader(path.open()))
    return list(rows[0].keys()), rows


def write_csv(path: Path, fields: list[str], rows: list[dict]) -> None:
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def with_external(scenario: str, mutate) -> set[str]:
    path = PHASE / "external" / f"{scenario}-fmpy-900.csv"
    fields, rows = load_csv(path)
    original = path.read_text()
    try:
        mutate(rows)
        write_csv(path, fields, rows)
        result = traj.compare(scenario)
        return {finding["class"] for finding in result["findings"]}
    finally:
        path.write_text(original)


def main() -> int:
    controls = []

    def add(name: str, expected: str, classes: set[str]) -> None:
        detected = expected in classes
        controls.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes)})
        print(name, "PASS" if detected else "FAIL", sorted(classes))

    def delay(rows):
        for row in rows:
            if float(row["failureAppliedTime_s"]) >= 0:
                row["failureAppliedTime_s"] = f"{float(row['failureAppliedTime_s']) + 3600.0:.9f}"
    add("pv-failure-delayed-1h", "EVENT_TIME_DISPLACEMENT", with_external("N8-01", delay))

    def missing(rows):
        for row in rows:
            if float(row["failureAppliedTime_s"]) >= 0:
                row["failureAppliedTime_s"] = "-1.000000000"
    add("pv-failure-missing", "EVENT_MISSING", with_external("N8-01", missing))

    def wrong_state(rows):
        for row in rows:
            if abs(float(row["time_s"]) - 7200.0) < 1e-6:
                row["pvAvailable"] = "1.000000000"
    add("pv-failure-state-wrong", "STATE_TRANSITION_MISMATCH", with_external("N8-01", wrong_state))

    def completion_delay(rows):
        for row in rows:
            if float(row["repairCompletedTime_s"]) >= 0:
                row["repairCompletedTime_s"] = f"{float(row['repairCompletedTime_s']) + 3600.0:.9f}"
    add("repair-completion-delayed", "EVENT_TIME_DISPLACEMENT", with_external("N8-01", completion_delay))

    def terminal_delay(rows):
        rows[-1]["lastEventTime_s"] = f"{float(rows[-1]['lastEventTime_s']) + 3600.0:.9f}"
    add("terminal-event-delayed", "EVENT_TIME_DISPLACEMENT", with_external("N8-04", terminal_delay))

    # Canonical event order mutation.
    events_path = PHASE / "event-observations/events.csv"
    original = events_path.read_text()
    rows = list(csv.DictReader(original.splitlines(), delimiter="\t"))
    for row in rows:
        if row["scenario"] == "N8-01" and row["event_sequence"] == "2":
            row["event_sequence"] = "3"
        elif row["scenario"] == "N8-01" and row["event_sequence"] == "3":
            row["event_sequence"] = "2"
    try:
        fields = list(rows[0].keys())
        with events_path.open("w", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=fields, delimiter="\t")
            writer.writeheader()
            writer.writerows(rows)
        classes = {finding["class"] for finding in traj.compare("N8-01")["findings"]}
    finally:
        events_path.write_text(original)
    add("canonical-event-order-swapped", "EVENT_ORDER_MISMATCH", classes)

    output = PHASE / "negative-controls/trajectory-results.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n8-trajectory-controls/1", "results": controls}, indent=2) + "\n")
    return 0 if all(entry["detected"] for entry in controls) else 1


if __name__ == "__main__":
    raise SystemExit(main())

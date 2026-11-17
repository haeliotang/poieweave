#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Mutation controls for N8 event identity and state transitions."""

from __future__ import annotations

import copy
import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n8_check", PHASE / "check.py")
check = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(check)


def load():
    rows = list(csv.DictReader((PHASE / "event-observations/events.csv").open(), delimiter="\t"))
    summary = list(csv.DictReader((PHASE / "event-observations/summary.csv").open(), delimiter="\t"))
    return rows, summary


def find(rows, scenario, seq, asset):
    return next(row for row in rows if row["scenario"] == scenario and row["event_sequence"] == seq
                and row["asset_id"] == asset)


def all_for(rows, scenario, seq):
    return [row for row in rows if row["scenario"] == scenario and row["event_sequence"] == seq]


def main() -> int:
    rows, summary = load()
    if check.validate(rows, summary):
        print("positive control rejected", file=__import__("sys").stderr)
        return 1
    controls = []

    def add(name, expected, mutate):
        mutated = (copy.deepcopy(rows), copy.deepcopy(summary))
        mutate(*mutated)
        detected = expected in check.validate(*mutated)
        controls.append({"control": name, "expected": expected, "detected": detected})
        print(name, "PASS" if detected else "FAIL")

    add("same-time-sequence-swap", "same-time event order",
        lambda r, s: [row.update(event_sequence="3") for row in all_for(r, "N8-03", "2")] +
        [row.update(event_sequence="2") for row in all_for(r, "N8-03", "3")])
    add("duplicate-event-id", "conflicting event id for sequence",
        lambda r, s: find(r, "N8-03", "3", "robot-m1").update(event_id="ev-0002-RepairCompleted"))
    add("skipped-event", "N8-03: non-contiguous sequence",
        lambda r, s: r.__setitem__(slice(None), [row for row in r if not (row["scenario"] == "N8-03" and row["event_sequence"] == "2")]))
    add("unprocessed-committed", "uncommitted event in accepted evidence",
        lambda r, s: find(r, "N8-03", "3", "robot-m1").update(event_status="NotApplied"))
    add("wrong-before-state", "completion state transition",
        lambda r, s: find(r, "N8-03", "2", "pump-p1").update(status_before="Available"))
    add("wrong-after-state", "failure state transition",
        lambda r, s: find(r, "N8-03", "3", "robot-m1").update(status_after="Available"))
    add("batch-after-as-event", "same-time event order",
        lambda r, s: [row.update(event_batch_sequence="2") for row in all_for(r, "N8-03", "3")])
    add("event-time-shift", "same-time event order",
        lambda r, s: [row.update(at_s="10799.000") for row in all_for(r, "N8-03", "2")] +
        [row.update(at_s="10799.000") for row in all_for(r, "N8-03", "3")])
    output = PHASE / "negative-controls/event-results.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n8-event-controls/1", "results": controls}, indent=2) + "\n")
    return 0 if all(entry["detected"] for entry in controls) else 1


if __name__ == "__main__":
    raise SystemExit(main())

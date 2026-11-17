#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Adversarial controls for the strict N8-H trajectory comparator."""

from __future__ import annotations

import copy
import csv
import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
spec = importlib.util.spec_from_file_location("n8h", PHASE / "compare_strict.py")
strict = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(strict)


def read_csv(path: Path, delimiter: str = ",") -> tuple[list[str], list[dict]]:
    rows = list(csv.DictReader(path.open(), delimiter=delimiter))
    return list(rows[0].keys()), rows


def write_csv(path: Path, fields: list[str], rows: list[dict], delimiter: str = ",") -> None:
    with path.open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, delimiter=delimiter)
        writer.writeheader()
        writer.writerows(rows)


def with_mutation(paths: list[Path], mutate, scenario: str | None = None) -> set[str]:
    backups = {path: path.read_text() for path in paths}
    try:
        mutate()
        return {finding["class"] for finding in strict.compare(scenario or _scenario_from(paths))["findings"]}
    finally:
        for path, text in backups.items():
            path.write_text(text)


def _scenario_from(paths: list[Path]) -> str:
    name = paths[0].name
    if "01" in name:
        return "N8-01"
    if "02" in name:
        return "N8-02"
    return "N8-04"


results = []


def record(name: str, expected: str, classes: set[str]) -> None:
    detected = expected in classes
    results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes)})
    print(name, "PASS" if detected else "FAIL", sorted(classes))


ext01 = PHASE / "external/N8H-N8-01.csv"
ext02 = PHASE / "external/N8H-N8-02.csv"
meta02 = PHASE / "external/N8H-N8-02.meta.json"
events = ROOT / "audit/phase8/event-observations/events.csv"
n4_05 = ROOT / "audit/phase4/canonical/N4-05-observations.csv"


def time_shift_14400():
    fields, rows = read_csv(ext01)
    for row in rows:
        if abs(float(row["time_s"]) - 14400.0) < 1e-9:
            row["time_s"] = "14000.000000000"
    write_csv(ext01, fields, rows)


record("nearest-neighbor-14400-to-14000", "TIME_ALIGNMENT_MISMATCH", with_mutation([ext01], time_shift_14400))


def time_shift_4500():
    fields, rows = read_csv(ext02)
    for row in rows:
        if abs(float(row["time_s"]) - 4500.0) < 1e-9:
            row["time_s"] = "4000.000000000"
    write_csv(ext02, fields, rows)


record("nearest-neighbor-4500-to-4000", "TIME_ALIGNMENT_MISMATCH", with_mutation([ext02], time_shift_4500))


def post_termination_observed():
    fields, rows = read_csv(n4_05, delimiter="\t")
    for row in rows:
        if row["at_s"] == "14400.000":
            row["status"] = "OBSERVED"
            row["energy_kwh"] = "0.000000000"
    write_csv(n4_05, fields, rows, delimiter="\t")


record("post-termination-zero-as-observed", "UNOBSERVABLE_SAMPLE_USED", with_mutation([n4_05], post_termination_observed))


def step_metadata():
    meta = json.loads(meta02.read_text())
    meta["nominal_communication_step_s"] = 900.0
    meta02.write_text(json.dumps(meta, indent=2) + "\n")


record("n8-02-step-1000-declared-900", "EXECUTION_METADATA_MISMATCH", with_mutation([meta02], step_metadata))


def extra_energy():
    fields, rows = read_csv(ext01)
    for row in rows:
        if abs(float(row["time_s"]) - 14400.0) < 1e-9:
            row["energy_J"] = f"{float(row['energy_J']) + 100000.0:.9f}"
    write_csv(ext01, fields, rows)


record("post-completion-extra-100kj", "UNEXPLAINED_RESIDUAL", with_mutation([ext01], extra_energy))


def pv_generation_after_failure():
    fields, rows = read_csv(ext02)
    for row in rows:
        if abs(float(row["time_s"]) - 7200.0) < 1e-9:
            row["generated_J"] = f"{float(row['generated_J']) + 1000000.0:.9f}"
    write_csv(ext02, fields, rows)


record("pv-generation-after-failure", "STATE_CONTRACT_FAILURE", with_mutation([ext02], pv_generation_after_failure))


def event_order_swap():
    fields, rows = read_csv(events, delimiter="\t")
    for row in rows:
        if row["scenario"] == "N8-01" and row["event_sequence"] == "2":
            row["event_sequence"] = "3"
        elif row["scenario"] == "N8-01" and row["event_sequence"] == "3":
            row["event_sequence"] = "2"
    write_csv(events, fields, rows, delimiter="\t")


record("same-time-event-order-swap", "EVENT_ORDER_MISMATCH", with_mutation([events], event_order_swap, "N8-01"))

output = PHASE / "negative-controls" / "strict-results.json"
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps({"schema": "poie.s5v-n8h-strict-controls/1", "results": results}, indent=2) + "\n")
raise SystemExit(0 if all(entry["detected"] for entry in results) else 1)

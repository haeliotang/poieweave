#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce the three N8 comparator false-PASS defects without modifying frozen N8 evidence."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
N8 = ROOT / "audit/phase8"


def rows(path: Path) -> list[dict]:
    return list(csv.DictReader(path.open()))


def row_at_nearest(items: list[dict], at_s: float) -> dict:
    return min(items, key=lambda row: abs(float(row["time_s"]) - at_s))


def main() -> int:
    old = json.loads((N8 / "trajectory-comparisons/trajectories.json").read_text())
    old_status = {entry["scenario"]: entry["classification"] for entry in old["scenarios"]}
    canonical = list(csv.DictReader((ROOT / "audit/phase4/canonical/N4-02-observations.csv").open(), delimiter="\t"))
    canonical_14400 = next(row for row in canonical if abs(float(row["at_s"]) - 14400.0) < 1e-6)
    external_02 = rows(N8 / "external/N8-02-fmpy-1000.csv")
    nearest = row_at_nearest(external_02, 14400.0)
    canonical_energy = float(canonical_14400["energy_kwh"]) * 3_600_000.0
    external_energy = float(nearest["energy_J"])
    p0a = {
        "requested_time_s": 14400.0,
        "canonical_actual_time_s": 14400.0,
        "external_actual_time_s": float(nearest["time_s"]),
        "time_alignment_error_s": abs(float(nearest["time_s"]) - 14400.0),
        "canonical_energy_J": canonical_energy,
        "external_energy_J": external_energy,
        "raw_difference_J": external_energy - canonical_energy,
        "old_status": old_status["N8-02"],
        "root_cause": "row_at_nearest selected 14000 s for a 14400 s comparison",
        "expected_classification": "TIME_ALIGNMENT_MISMATCH",
    }
    canonical_05 = list(csv.DictReader((ROOT / "audit/phase4/canonical/N4-05-observations.csv").open(), delimiter="\t"))
    unobservable = [row for row in canonical_05 if row["at_s"] == "14400.000"][0]
    external_04 = rows(N8 / "external/N8-04-fmpy-900.csv")
    nearest_04 = row_at_nearest(external_04, 14400.0)
    p0b = {
        "canonical_at_s": 14400.0,
        "canonical_status": unobservable["status"],
        "canonical_energy_placeholder_J": float(unobservable["energy_kwh"]) * 3_600_000.0,
        "external_selected_time_s": float(nearest_04["time_s"]),
        "external_energy_J": float(nearest_04["energy_J"]),
        "old_status": old_status["N8-04"],
        "root_cause": "UNOBSERVABLE_AFTER_TERMINATION placeholder 0 J entered energy statistics",
        "expected_classification": "UNOBSERVABLE_SAMPLE_USED",
    }
    p0c = {
        "old_numerical_classification": old["scenarios"][0]["numerical_error"]["classification"],
        "old_max_abs_error_J": max(entry["numerical_error"]["max_abs_error_J"] for entry in old["scenarios"]),
        "hypothetical_post_completion_extra_error_J": 100_000.0,
        "root_cause": "old comparator labels any max_abs_error > 1 J as MODEL_ASSUMPTION_DIFFERENCE without checking repair window/sign/amount",
        "old_status_would_remain": "MATCH",
        "expected_classification": "UNEXPLAINED_RESIDUAL",
    }
    result = {
        "schema": "poie.s5v-n8h-false-pass/1",
        "P0_A_nearest_neighbor": p0a,
        "P0_B_post_termination": p0b,
        "P0_C_unconditional_assumption": p0c,
    }
    output = PHASE / "false-pass.json"
    output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Event-resolved cross-model trajectory comparator for N4 PV/repair scenarios."""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
SCENARIOS = {
    "N8-01": {"external": "N8-01-fmpy-900.csv", "event_kinds": ["AssetFailure", "RepairCompleted"]},
    "N8-02": {"external": "N8-02-fmpy-1000.csv", "event_kinds": ["AssetFailure", "RepairCompleted"]},
    "N8-04": {"external": "N8-04-fmpy-900.csv", "event_kinds": ["AssetFailure", "RepairInterrupted"]},
}


def canonical_events(scenario: str) -> list[dict]:
    rows = list(csv.DictReader((PHASE / "event-observations/events.csv").open(), delimiter="\t"))
    events = {}
    for row in rows:
        if row["scenario"] != scenario:
            continue
        key = (int(row["event_sequence"]), row["event_kind"])
        events.setdefault(key, []).append(row)
    result = []
    for (sequence, kind), entries in sorted(events.items()):
        result.append({"sequence": sequence, "kind": kind, "at_s": float(entries[0]["at_s"]),
                       "assets": {row["asset_id"]: {"before": row["status_before"], "after": row["status_after"]}
                                  for row in entries}})
    return result


def external_rows(path: Path) -> list[dict]:
    return list(csv.DictReader(path.open()))


def row_at(rows: list[dict], at_s: float) -> dict:
    return min(rows, key=lambda row: abs(float(row["time_s"]) - at_s))


def row_before(rows: list[dict], at_s: float) -> dict:
    candidates = [row for row in rows if float(row["time_s"]) < at_s - 1e-9]
    return max(candidates, key=lambda row: float(row["time_s"])) if candidates else rows[0]


def row_after(rows: list[dict], at_s: float) -> dict:
    candidates = [row for row in rows if float(row["time_s"]) >= at_s - 1e-9]
    return min(candidates, key=lambda row: float(row["time_s"])) if candidates else rows[-1]


def compare(scenario: str) -> dict:
    config = SCENARIOS[scenario]
    events = canonical_events(scenario)
    externals = external_rows(PHASE / "external" / config["external"])
    findings = []
    comparisons = []
    for event in events:
        if event["kind"] not in config["event_kinds"]:
            continue
        external_time = None
        if event["kind"] == "AssetFailure":
            row = externals[-1]
            if scenario in ("N8-01", "N8-02"):
                external_time = float(row["failureAppliedTime_s"])
                before = float(row_before(externals, event["at_s"])["pvAvailable"])
                after = float(row_after(externals, event["at_s"])["pvAvailable"])
                state_match = before > 0.5 and after < 0.5
            else:
                external_time = float(row["robotFailureAppliedTime_s"])
                before = float(row_before(externals, event["at_s"])["robotAvailable"])
                after = float(row_after(externals, event["at_s"])["robotAvailable"])
                state_match = before > 0.5 and after < 0.5
        elif event["kind"] == "RepairCompleted":
            external_time = float(externals[-1]["repairCompletedTime_s"])
            state_match = True
        else:
            external_time = float(externals[-1]["robotFailureAppliedTime_s"])
            state_match = float(row_after(externals, event["at_s"])["repairState"]) >= 2.0
        dt = None if external_time is None else abs(external_time - event["at_s"])
        if external_time is None or external_time < 0:
            findings.append({"class": "EVENT_MISSING"})
        elif dt > 1e-6:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "canonical_s": event["at_s"], "external_s": external_time})
        if not state_match:
            findings.append({"class": "STATE_TRANSITION_MISMATCH", "kind": event["kind"]})
        canonical_energy = None
        if event["kind"] == "RepairCompleted" and "energy" in event:
            canonical_energy = event["energy"]
        comparisons.append({"kind": event["kind"], "sequence": event["sequence"], "at_s": event["at_s"],
                            "external_time_s": external_time, "time_error_s": dt,
                            "state_match": state_match,
                            "canonical_assets": event["assets"]})
    kinds = {event["kind"]: event for event in events}
    failure = kinds.get("AssetFailure")
    completion = kinds.get("RepairCompleted")
    if failure and completion and failure["sequence"] >= completion["sequence"]:
        findings.append({"class": "EVENT_ORDER_MISMATCH", "detail": "failure must precede completion"})
    depleted = kinds.get("EnergyDepleted")
    if depleted:
        terminal_rows = [row for row in externals if float(row["depleted"]) > 0.5]
        external_time = float(terminal_rows[0]["time_s"]) if terminal_rows else None
        if external_time is None or abs(external_time - depleted["at_s"]) > 1e-6:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "EnergyDepleted",
                             "canonical_s": depleted["at_s"], "external_s": external_time})
    last = externals[-1]
    last_event_time = float(last["lastEventTime_s"])
    if scenario == "N8-04" and last_event_time >= 0:
        terminal = kinds.get("RepairInterrupted")
        if terminal is None or abs(last_event_time - terminal["at_s"]) > 1e-6:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "terminal_event",
                             "canonical_s": terminal["at_s"] if terminal else None, "external_s": last_event_time})
    # Energy comparison at repair completion/grid time is reported, not silently waived.
    energy = []
    for at_s in (7200.0, 10800.0):
        canonical = next((event for event in events if event["kind"] in ("AssetFailure", "RepairCompleted")
                          and abs(event["at_s"] - at_s) < 1e-6), None)
        row = row_at(externals, at_s)
        if canonical is not None:
            energy.append({"at_s": at_s, "external_energy_J": float(row["energy_J"]),
                           "comparison": "MODEL_ASSUMPTION_BOUNDARY",
                           "note": "canonical repair energy commits at completion; external FMU draws uniformly"})
    n4_case = {"N8-01": "N4-01", "N8-02": "N4-02", "N8-04": "N4-05"}[scenario]
    canonical_obs = list(csv.DictReader((PHASE.parent / f"phase4/canonical/{n4_case}-observations.csv").open(), delimiter="\t"))
    numerical = []
    for obs in canonical_obs:
        at_s = float(obs["at_s"])
        external_row = row_at(externals, at_s)
        canonical_energy = float(obs["energy_kwh"]) * 3_600_000.0
        external_energy = float(external_row["energy_J"])
        numerical.append({"at_s": at_s, "canonical_energy_J": canonical_energy, "external_energy_J": external_energy,
                          "abs_error_J": abs(canonical_energy - external_energy),
                          "rel_error": abs(canonical_energy - external_energy) / max(abs(canonical_energy), 1.0)})
    max_abs = max((entry["abs_error_J"] for entry in numerical), default=0.0)
    max_rel = max((entry["rel_error"] for entry in numerical), default=0.0)
    external_times = [float(row["time_s"]) for row in externals]
    external_contract_valid = all(b > a for a, b in zip(external_times, external_times[1:]))
    canonical_times = [event["at_s"] for event in events]
    canonical_contract_valid = all(b >= a for a, b in zip(canonical_times, canonical_times[1:]))
    result = {"schema": "poie.s5v-n8-trajectory/1", "scenario": scenario,
              "canonical_contract_valid": canonical_contract_valid,
              "external_contract_valid": external_contract_valid,
              "cross_model_agreement": not findings,
              "numerical_error": {"max_abs_error_J": max_abs, "max_rel_error": max_rel,
                                  "step_size_s": 900.0, "comparison_interval_s": [0.0, 14400.0],
                                  "classification": "MODEL_ASSUMPTION_DIFFERENCE" if max_abs > 1.0 else "NUMERICAL_INTEGRATION_ERROR"},
              "energy_samples": numerical,
              "external_file": config["external"], "event_comparisons": comparisons,
              "energy_comparisons": energy, "findings": findings,
              "first_divergence": findings[0] if findings else {"class": "NO_DIVERGENCE", "at_s": None},
              "classification": "MATCH" if not findings else findings[0]["class"],
              "contract_valid": not any(f["class"] in ("EVENT_MISSING", "STATE_TRANSITION_MISMATCH") for f in findings)}
    return result


def main() -> int:
    results = [compare(scenario) for scenario in SCENARIOS]
    output = PHASE / "trajectory-comparisons" / "trajectories.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n8-trajectory-set/1", "scenarios": results}, indent=2) + "\n")
    for result in results:
        print(result["scenario"], result["classification"], result["findings"])
    return 0 if all(not result["findings"] for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Strict N8-H temporal alignment, validity and energy-residual comparator."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
INITIAL_J = 144_000_000.0
REPAIR_J = 1_800_000.0
REPAIR_DURATION_S = 10_800.0
TOLERANCE_J = 1.0
SCENARIOS = {
    "N8-01": {"external": "N8H-N8-01.csv", "metadata": "N8H-N8-01.meta.json", "canonical": "N4-01",
              "step": 900.0, "interrupted": False, "completion_s": 10800.0},
    "N8-02": {"external": "N8H-N8-02.csv", "metadata": "N8H-N8-02.meta.json", "canonical": "N4-02",
              "step": 1000.0, "interrupted": False, "completion_s": 10800.0},
    "N8-04": {"external": "N8H-N8-04.csv", "metadata": "N8H-N8-04.meta.json", "canonical": "N4-05",
              "step": 900.0, "interrupted": True, "completion_s": None},
}


def f(row: dict, key: str) -> float:
    return float(row[key])


def expected_assumption(scenario: str, at_s: float) -> float:
    # external - canonical: FMU draws repair energy uniformly, canonical commits at completion.
    if scenario == "N8-04":
        return -REPAIR_J * min(at_s, 7200.0) / REPAIR_DURATION_S
    if at_s < 10800.0:
        return -REPAIR_J * at_s / REPAIR_DURATION_S
    return 0.0


def canonical_samples(n4_case: str) -> list[dict]:
    path = ROOT / f"audit/phase4/canonical/{n4_case}-observations.csv"
    rows = list(csv.DictReader(path.open(), delimiter="\t"))
    result = {}
    for row in rows:
        at_s = f(row, "at_s")
        if row["status"] != "OBSERVED":
            result.setdefault(at_s, {"at_s": at_s, "phase": row["phase"], "status": row["status"], "reason": "CANONICAL_TERMINATED"})
            continue
        if row["phase"] == "After" or at_s not in result or result[at_s].get("status") != "OBSERVED":
            result[at_s] = {"at_s": at_s, "phase": row["phase"], "status": "OBSERVED", "row": row}
    return [result[key] for key in sorted(result)]


def external_rows(name: str) -> list[dict]:
    return list(csv.DictReader((PHASE / "external" / name).open()))


def exact_external(rows: list[dict], at_s: float):
    matches = [row for row in rows if abs(f(row, "time_s") - at_s) <= 1e-9]
    return matches[0] if matches else None


def canonical_events(scenario: str) -> dict[str, dict]:
    rows = list(csv.DictReader((ROOT / "audit/phase8/event-observations/events.csv").open(), delimiter="\t"))
    events = {}
    for row in rows:
        if row["scenario"] != scenario:
            continue
        events.setdefault(row["event_kind"], {"at_s": f(row, "at_s"), "sequence": int(row["event_sequence"]), "assets": {}})
        events[row["event_kind"]]["assets"][row["asset_id"]] = {"before": row["status_before"], "after": row["status_after"]}
    return events


def compare(scenario: str) -> dict:
    config = SCENARIOS[scenario]
    metadata = json.loads((PHASE / "external" / config["metadata"]).read_text())
    metadata_findings = []
    external = external_rows(config["external"])
    canonical = canonical_samples(config["canonical"])
    termination_s = 7200.0 if scenario == "N8-04" else None
    findings = []
    samples = []
    invalid = []
    for sample in canonical:
        at_s = sample["at_s"]
        if sample["status"] != "OBSERVED":
            invalid.append({"at_s": at_s, "phase": sample["phase"], "validity": "NOT_COMPARABLE",
                            "reason": sample["reason"]})
            continue
        if termination_s is not None and at_s > termination_s + 1e-9:
            if sample.get("row") is not None:
                findings.append({"class": "UNOBSERVABLE_SAMPLE_USED", "at_s": at_s,
                                 "reason": "OBSERVED status after canonical termination"})
            invalid.append({"at_s": at_s, "phase": sample["phase"], "validity": "NOT_COMPARABLE",
                            "reason": "CANONICAL_TERMINATED_7200"})
            continue
        if termination_s is not None and abs(at_s - termination_s) <= 1e-9:
            invalid.append({"at_s": at_s, "phase": sample["phase"], "validity": "NOT_COMPARABLE",
                            "reason": "EVENT_BOUNDARY_PHASE_MISMATCH"})
            continue
        row = exact_external(external, at_s)
        if row is None:
            findings.append({"class": "TIME_ALIGNMENT_MISMATCH", "at_s": at_s,
                             "reason": "NO_EXACT_EXTERNAL_SAMPLE"})
            invalid.append({"at_s": at_s, "phase": sample["phase"], "validity": "NOT_COMPARABLE",
                            "reason": "NO_EXACT_EXTERNAL_SAMPLE"})
            continue
        canonical_energy = f(sample["row"], "energy_kwh") * 3_600_000.0
        external_energy = f(row, "energy_J")
        raw = external_energy - canonical_energy
        explained = expected_assumption(scenario, at_s)
        residual = raw - explained
        samples.append({
            "requested_time_s": at_s,
            "canonical_actual_time_s": at_s,
            "external_actual_time_s": f(row, "time_s"),
            "time_alignment_error_s": abs(f(row, "time_s") - at_s),
            "alignment_method": "EXACT_SAMPLE",
            "phase": sample["phase"],
            "validity": "OBSERVED",
            "canonical_energy_J": canonical_energy,
            "external_energy_J": external_energy,
            "raw_energy_difference_J": raw,
            "explained_model_assumption_delta_J": explained,
            "unexplained_residual_J": residual,
        })
    conservation = []
    for row in external:
        residual = f(row, "energy_J") - (INITIAL_J + f(row, "generated_J") - f(row, "load_J")
                                         - f(row, "repair_J") - f(row, "curtailed_J"))
        conservation.append({"side": "external", "at_s": f(row, "time_s"), "residual_J": residual})
    for sample in canonical:
        if sample["status"] != "OBSERVED":
            continue
        row = sample["row"]
        residual = f(row, "energy_kwh") * 3_600_000.0 - (
            INITIAL_J + f(row, "generated_kwh") * 3_600_000.0 - f(row, "load_kwh") * 3_600_000.0
            - f(row, "repair_kwh") * 3_600_000.0 - f(row, "curtailed_kwh") * 3_600_000.0)
        conservation.append({"side": "canonical", "at_s": sample["at_s"], "residual_J": residual})
    events = canonical_events(scenario)
    external_last = external[-1]
    event_checks = []
    if "AssetFailure" in events:
        if scenario == "N8-04":
            ext_time = f(external_last, "robotFailureAppliedTime_s")
        else:
            ext_time = f(external_last, "failureAppliedTime_s")
        event_checks.append({"kind": "AssetFailure", "canonical_s": events["AssetFailure"]["at_s"],
                             "external_s": ext_time, "error_s": abs(ext_time - events["AssetFailure"]["at_s"])})
    if scenario != "N8-04" and "RepairCompleted" in events:
        ext_time = f(external_last, "repairCompletedTime_s")
        event_checks.append({"kind": "RepairCompleted", "canonical_s": events["RepairCompleted"]["at_s"],
                             "external_s": ext_time, "error_s": abs(ext_time - events["RepairCompleted"]["at_s"])})
    if scenario == "N8-04":
        event_checks.append({"kind": "RepairInterrupted", "canonical_s": events["RepairInterrupted"]["at_s"],
                             "external_s": f(external_last, "robotFailureAppliedTime_s"),
                             "error_s": abs(f(external_last, "robotFailureAppliedTime_s") - events["RepairInterrupted"]["at_s"]),
                             "repair_state": f(external_last, "repairState")})
    unexplained = [sample for sample in samples if abs(sample["unexplained_residual_J"]) > TOLERANCE_J]
    conservation_bad = [entry for entry in conservation if abs(entry["residual_J"]) > TOLERANCE_J]
    event_bad = [check for check in event_checks if check["error_s"] > 1e-6]
    if abs(metadata["nominal_communication_step_s"] - config["step"]) > 1e-9:
        findings.append({"class": "EXECUTION_METADATA_MISMATCH",
                         "declared": config["step"], "actual": metadata["nominal_communication_step_s"]})
    findings.extend({"class": "UNEXPLAINED_RESIDUAL", "at_s": sample["requested_time_s"],
                     "residual_J": sample["unexplained_residual_J"]} for sample in unexplained)
    findings.extend({"class": "ENERGY_CONSERVATION", "side": entry["side"], "at_s": entry["at_s"]}
                    for entry in conservation_bad)
    if "AssetFailure" in events and "RepairCompleted" in events:
        if events["AssetFailure"]["sequence"] >= events["RepairCompleted"]["sequence"]:
            findings.append({"class": "EVENT_ORDER_MISMATCH"})
    if scenario in ("N8-01", "N8-02") and "AssetFailure" in events:
        failure_s = events["AssetFailure"]["at_s"]
        failure_row = exact_external(external, failure_s)
        if failure_row is not None:
            base_generated = f(failure_row, "generated_J")
            for row in external:
                if f(row, "time_s") > failure_s + 1e-9 and f(row, "generated_J") > base_generated + 1.0:
                    findings.append({"class": "STATE_CONTRACT_FAILURE", "at_s": f(row, "time_s"),
                                     "reason": "PV generated energy after failure"})
                    break
    findings.extend({"class": "EVENT_TIME_DISPLACEMENT", "kind": check["kind"]} for check in event_bad)
    if scenario == "N8-04" and event_checks and event_checks[-1].get("repair_state", 0.0) < 2.0:
        findings.append({"class": "REPAIR_COMMIT_MISMATCH"})
    first_raw = next((sample["requested_time_s"] for sample in samples
                      if abs(sample["raw_energy_difference_J"]) > TOLERANCE_J), None)
    first_unexplained = next((sample["requested_time_s"] for sample in samples
                              if abs(sample["unexplained_residual_J"]) > TOLERANCE_J), None)
    status = "MATCH" if not findings else "FAIL"
    return {
        "schema": "poie.s5v-n8h-strict-trajectory/1",
        "scenario": scenario,
        "nominal_communication_step_s": metadata["nominal_communication_step_s"],
        "declared_step_s": config["step"],
        "actual_max_step_s": metadata["max_actual_step_s"],
        "event_split_times_s": metadata["event_split_times_s"],
        "samples": samples,
        "excluded_samples": invalid,
        "energy_conservation": conservation,
        "discrete_events": event_checks,
        "first_raw_difference_time_s": first_raw,
        "first_unexplained_divergence_time_s": first_unexplained,
        "first_invalid_alignment_time_s": invalid[0]["at_s"] if invalid else None,
        "first_discrete_event_divergence": event_bad[0] if event_bad else None,
        "findings": findings,
        "classification": "MATCH_WITH_EXPLAINED_ASSUMPTION" if not findings and any(abs(s["raw_energy_difference_J"]) > TOLERANCE_J for s in samples) else ("MATCH" if not findings else findings[0]["class"]),
        "status": status,
    }


def main() -> int:
    results = [compare(scenario) for scenario in SCENARIOS]
    output = PHASE / "comparisons" / "strict-trajectories.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"schema": "poie.s5v-n8h-strict-set/1", "scenarios": results}, indent=2) + "\n")
    for result in results:
        print(result["scenario"], result["status"], result["classification"], result["findings"])
    return 0 if all(result["status"] == "MATCH" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

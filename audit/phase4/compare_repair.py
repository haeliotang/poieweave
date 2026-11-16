#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N4-C repair-interruption comparator (N4-05)."""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
REPAIR_ENERGY_J = 1_800_000.0
DURATION_S = 10_800.0
TOL_J = 1.0


def rows(path: Path) -> list[dict[str, float]]:
    with path.open(newline="") as handle:
        return [{key: float(value) for key, value in row.items()} for row in csv.DictReader(handle)]


def compare(fmu_path: Path) -> dict:
    canonical = json.loads((PHASE / "canonical/N4-05-summary.json").read_text())
    data = rows(fmu_path)
    last = data[-1]
    failure_s = float(last["robotFailureAppliedTime_s"])
    terminal_s = float(last["time_s"])
    fmu_energy = float(last["energy_J"])
    canonical_energy = canonical["final_energy_kwh"] * 3_600_000.0
    expected_delta = -REPAIR_ENERGY_J * min(failure_s, DURATION_S) / DURATION_S
    residual = (fmu_energy - canonical_energy) - expected_delta
    findings = []
    if failure_s != 7200.0:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT", "canonical_s": 7200.0, "fmu_s": failure_s})
    if terminal_s != 7200.0:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "terminal", "canonical_s": 7200.0, "fmu_s": terminal_s})
    if float(last["repairState"]) != 2.0:
        findings.append({"class": "STATE_TRANSITION_MISMATCH", "kind": "repair_state", "fmu": last["repairState"]})
    if float(last["robotAvailable"]) != 0.0:
        findings.append({"class": "STATE_TRANSITION_MISMATCH", "kind": "robot_availability", "fmu": last["robotAvailable"]})
    if float(last["repairCompletedTime_s"]) >= 0.0:
        findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "kind": "repair committed after resource failure"})
    if abs(residual) > TOL_J:
        findings.append({"class": "UNEXPLAINED", "residual_J": residual})
    first = findings[0] if findings else {"class": "MODEL_ASSUMPTION_DIFFERENCE", "at_s": 3600.0}
    result = {
        "schema": "poie.s5v-n4c-repair/1",
        "case": "N4-05",
        "canonical": {"termination": canonical["termination"], "survived_s": canonical["survived_s"],
                       "final_energy_J": canonical_energy, "repair_completed": False},
        "fmu": {"terminal_s": terminal_s, "robot_failure_s": failure_s, "repair_state": float(last["repairState"]),
                "robot_available": float(last["robotAvailable"]), "repair_completed": float(last["repairCompletedTime_s"]) >= 0.0,
                "final_energy_J": fmu_energy},
        "model_assumption": {"expression": "FMU uniform partial draw until interruption",
                             "expected_delta_J": expected_delta, "actual_delta_J": fmu_energy - canonical_energy,
                             "residual_J": residual},
        "findings": findings,
        "first_divergence": first,
        "status": "UNEXPLAINED" if any(f["class"] == "UNEXPLAINED" for f in findings) else "EXPLAINED",
    }
    return result


def main() -> int:
    result = compare(PHASE / "repair/N4-05-fmpy-900.csv")
    output = PHASE / "repair" / "comparison.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n")
    print("N4-05", result["status"], result["first_divergence"]["class"],
          "residual", result["model_assumption"]["residual_J"])
    return 0 if result["status"] == "EXPLAINED" else 1


if __name__ == "__main__":
    raise SystemExit(main())

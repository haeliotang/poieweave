#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N4 event-level cross-model comparator with first-divergence classification."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PHASE = ROOT / "audit/phase4"
CONFIG = json.loads((PHASE / "config.json").read_text())
SAMPLES = CONFIG["samples_s"]
REPAIR_ENERGY_J = CONFIG["repair"]["energy_kwh"] * 3_600_000.0
REPAIR_DURATION_S = CONFIG["repair"]["duration_s"]
TOL_J = CONFIG["tolerance"]["energy_j"]
TOL_T = CONFIG["tolerance"]["event_time_s"]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_csv(path: Path, delimiter: str = ",") -> list[dict[str, float | str]]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle, delimiter=delimiter))


def canonical_rows(case: str) -> dict:
    observations = read_csv(PHASE / "canonical" / f"{case}-observations.csv", "\t")
    events = read_csv(PHASE / "canonical" / f"{case}-events.csv", "\t")
    summary = json.loads((PHASE / "canonical" / f"{case}-summary.json").read_text())
    energy = {}
    event_energy = {}
    for row in observations:
        if row["phase"] == "After" and row["status"] == "OBSERVED":
            energy[float(row["at_s"])] = float(row["energy_kwh"]) * 3_600_000.0
        if row["phase"] in ("Before", "After") and row["status"] == "OBSERVED":
            event_energy[(float(row["at_s"]), row["phase"])] = float(row["energy_kwh"]) * 3_600_000.0
    failure = next((row for row in events if row["kind"].startswith("failure:pv-1")), None)
    repair = next((row for row in events if row["kind"].startswith("repair_completed")), None)
    return {"energy": energy, "event_energy": event_energy, "failure": failure, "repair": repair,
            "summary": summary, "events": events}


def fmu_rows(path: Path) -> dict:
    rows = read_csv(path)
    energy = {float(row["time_s"]): float(row["energy_J"]) for row in rows}
    last = rows[-1]
    return {
        "energy": energy,
        "terminal_s": float(last["time_s"]),
        "terminal_energy_J": float(last["energy_J"]),
        "generated_J": float(last["generated_J"]),
        "repair_J": float(last["repair_J"]),
        "failure_applied_s": float(last["failureAppliedTime_s"]),
        "terminated": int(float(last["terminated"])),
        "pv_failed": any(float(row["pvAvailable"]) == 0.0 for row in rows),
        "repair_completed_s": next(
            (float(row["time_s"]) for row in rows if float(row["repair_J"]) >= REPAIR_ENERGY_J - 1.0), None
        ),
        "last_event_code": float(last["lastEventCode"]),
        "last_event_time_s": float(last["lastEventTime_s"]),
        "repair_completed_output_s": float(last["repairCompletedTime_s"]),
        "rows": rows,
    }


def conservation(rows: list[dict]) -> float:
    worst = 0.0
    for left, right in zip(rows, rows[1:]):
        dgen = float(right["generated_J"]) - float(left["generated_J"])
        dload = float(right["load_J"]) - float(left["load_J"])
        drepair = float(right["repair_J"]) - float(left["repair_J"])
        dcurtail = float(right["curtailed_J"]) - float(left["curtailed_J"])
        denergy = float(right["energy_J"]) - float(left["energy_J"])
        worst = max(worst, abs(denergy - (dgen - dload - drepair - dcurtail)))
    return worst

def compare_case(case: str, fmu_path: Path) -> dict:
    canonical = canonical_rows(case)
    fmu = fmu_rows(fmu_path)
    failure_s = next(entry["failure_s"] for entry in CONFIG["cases"] if entry["id"] == case)
    findings = []
    # 1. expected repair-assumption difference
    max_residual = 0.0
    first_assumption = None
    for at_s in SAMPLES:
        if at_s not in canonical["energy"]:
            findings.append({"at_s": at_s, "class": "OBSERVATION_ARTIFACT", "reason": "canonical sample unavailable"})
            continue
        fmu_energy = fmu["energy"].get(float(at_s))
        if fmu_energy is None:
            findings.append({"at_s": at_s, "class": "OBSERVATION_ARTIFACT", "reason": "fmu sample unavailable"})
            continue
        expected = -REPAIR_ENERGY_J * min(at_s, REPAIR_DURATION_S) / REPAIR_DURATION_S if at_s < REPAIR_DURATION_S else 0.0
        actual = fmu_energy - canonical["energy"][at_s]
        residual = actual - expected
        max_residual = max(max_residual, abs(residual))
        if first_assumption is None and abs(actual) > TOL_J:
            first_assumption = {"at_s": at_s, "class": "MODEL_ASSUMPTION_DIFFERENCE",
                                "fmu_minus_canonical_J": actual, "expected_J": expected}
        if abs(residual) > TOL_J:
            findings.append({"at_s": at_s, "class": "UNEXPLAINED", "actual_J": actual, "expected_J": expected,
                             "residual_J": residual})
    # 2. event time / state
    canonical_failure = float(canonical["failure"]["at_s"]) if canonical["failure"] else None
    if failure_s is not None:
        if abs(fmu["failure_applied_s"] - canonical_failure) > TOL_T:
            findings.append({"at_s": canonical_failure, "class": "EVENT_TIME_DISPLACEMENT",
                             "canonical_s": canonical_failure, "fmu_s": fmu["failure_applied_s"]})
        before = canonical["event_energy"].get((canonical_failure, "Before"))
        after = canonical["event_energy"].get((canonical_failure, "After"))
        fmu_event = fmu["energy"].get(float(canonical_failure))
        if before is not None and after is not None and fmu_event is not None:
            expected = -REPAIR_ENERGY_J * min(canonical_failure, REPAIR_DURATION_S) / REPAIR_DURATION_S \
                if canonical_failure < REPAIR_DURATION_S else 0.0
            residual = (fmu_event - after) - expected
            if abs(residual) > TOL_J:
                findings.append({"at_s": canonical_failure, "class": "UNEXPLAINED", "kind": "event_after",
                                 "residual_J": residual, "canonical_after_J": after, "fmu_J": fmu_event})
        else:
            findings.append({"at_s": canonical_failure, "class": "OBSERVATION_ARTIFACT",
                             "reason": "event state unavailable"})
        if not fmu["pv_failed"]:
            findings.append({"at_s": canonical_failure, "class": "STATE_TRANSITION_MISMATCH",
                             "reason": "FMU pvAvailable never became 0"})
        if abs(fmu["terminal_s"] - canonical["summary"]["survived_s"]) > TOL_T:
            findings.append({"at_s": canonical["summary"]["survived_s"], "class": "EVENT_TIME_DISPLACEMENT",
                             "kind": "terminal", "canonical_s": canonical["summary"]["survived_s"],
                             "fmu_s": fmu["terminal_s"]})
        # PV generation after failure must be zero in the FMU interval ending at/after failure
        after_failure = [row for row in fmu["rows"] if float(row["time_s"]) >= fmu["failure_applied_s"] + 1e-9]
        if after_failure:
            generated_later = float(after_failure[-1]["generated_J"]) - float(after_failure[0]["generated_J"])
            if generated_later > TOL_J:
                findings.append({"at_s": fmu["failure_applied_s"], "class": "RESOURCE_LEDGER_DIVERGENCE",
                                 "reason": "generation after PV failure", "generated_J": generated_later})
    else:
        if fmu["pv_failed"]:
            findings.append({"at_s": 0.0, "class": "STATE_TRANSITION_MISMATCH",
                             "reason": "FMU PV failed in control case"})
    canonical_repair = float(canonical["repair"]["at_s"]) if canonical["repair"] else None
    if canonical["events"]:
        last_kind = canonical["events"][-1]["kind"]
        canonical_code = 1.0 if last_kind.startswith("failure:") else 2.0 if last_kind.startswith("repair_completed") else None
        if canonical_code is not None and abs(fmu["last_event_code"] - canonical_code) > 0.5:
            findings.append({"at_s": fmu["last_event_time_s"], "class": "EVENT_ORDER_MISMATCH",
                             "canonical_last_event": last_kind, "canonical_code": canonical_code,
                             "fmu_code": fmu["last_event_code"]})
    if canonical_repair is not None and fmu["repair_completed_s"] is not None:
        if abs(fmu["repair_completed_s"] - canonical_repair) > TOL_T:
            findings.append({"at_s": canonical_repair, "class": "EVENT_TIME_DISPLACEMENT",
                             "kind": "repair_completion", "canonical_s": canonical_repair,
                             "fmu_s": fmu["repair_completed_s"]})
    # 3. conservation
    worst_conservation = conservation(fmu["rows"])
    if worst_conservation > CONFIG["tolerance"]["conservation_j"]:
        findings.append({"at_s": fmu["terminal_s"], "class": "RESOURCE_LEDGER_DIVERGENCE",
                         "reason": "energy conservation", "residual_J": worst_conservation})
    unexplained = [finding for finding in findings if finding["class"] == "UNEXPLAINED"]
    if unexplained:
        first = min(unexplained, key=lambda finding: finding["at_s"])
    elif findings:
        first = min(findings, key=lambda finding: finding["at_s"])
    elif first_assumption:
        first = first_assumption
    else:
        first = {"class": "NO_DIVERGENCE", "at_s": None}
    return {
        "case": case,
        "canonical": {"termination": canonical["summary"]["termination"], "survived_s": canonical["summary"]["survived_s"],
                       "failure_s": canonical_failure},
        "fmu": {"terminal_s": fmu["terminal_s"], "failure_s": fmu["failure_applied_s"], "pv_failed": fmu["pv_failed"],
                "generated_J": fmu["generated_J"], "repair_J": fmu["repair_J"],
                "repair_completed_s": fmu["repair_completed_s"]},
        "first_divergence": first,
        "findings": findings,
        "max_unexplained_residual_J": max_residual,
        "worst_conservation_residual_J": worst_conservation,
        "fmu_sha256": sha(fmu_path),
        "status": "UNEXPLAINED" if findings else "EXPLAINED",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmu-dir", default=str(PHASE / "results"))
    parser.add_argument("--output", default=str(PHASE / "differences" / "comparison.json"))
    args = parser.parse_args()
    results = []
    for case in [entry["id"] for entry in CONFIG["cases"] if entry.get("failure_asset", "pv-1") == "pv-1"]:
        path = Path(args.fmu_dir) / f"{case}-fmpy-900.csv"
        if not path.is_file():
            print(f"missing {path}", file=__import__("sys").stderr)
            return 1
        results.append(compare_case(case, path))
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"schema": "poie.s5v-n4-comparison/1", "cases": results}, indent=2) + "\n")
    for result in results:
        print(result["case"], result["status"], result["first_divergence"]["class"], result["first_divergence"].get("at_s"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

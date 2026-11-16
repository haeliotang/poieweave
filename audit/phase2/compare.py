#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Phase 2 closure: external FMU execution, analytic reference, canonical boundaries.

Important: the canonical engine currently commits repair energy when the
repair completes. The FMU spreads the same repair energy over the repair
window. Therefore this comparator does NOT claim trajectory-level agreement
during the repair window. It compares:
  1. FMU vs an independent closed-form analytic reference (uniform repair draw),
  2. FMU vs canonical at the exported boundaries (t=0, repair completion,
     energy depletion),
  3. C harness vs an independent fmpy importer.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
FMU = ROOT / "external/fmu-pv-battery/poie-pv-battery.fmu"
TAMPERED = ROOT / "external/fmu-pv-battery/poie-pv-battery-tampered.fmu"
HARNESS = ROOT / "external/fmu-pv-battery/build/run_fmu"
CONFIG = HERE / "config.json"
CANONICAL = HERE / "canonical-reference.txt"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_harness(fmu: Path, step: int, tag: str) -> Path:
    csv = HERE / "results" / f"{tag}.csv"
    log = HERE / "logs" / f"{tag}.log"
    result = subprocess.run(
        [str(HARNESS), str(fmu), str(step), str(csv)],
        cwd=ROOT / "external/fmu-pv-battery",
        capture_output=True,
        text=True,
        check=False,
    )
    log.write_text(result.stdout + result.stderr)
    if result.returncode != 0:
        raise RuntimeError(f"harness {tag} failed: {result.stderr[-500:]}")
    return csv


def parse(csv: Path) -> list[dict]:
    rows = []
    for line in csv.read_text().strip().splitlines()[1:]:
        t, energy, generated, load, repair, curtailed, depleted, terminated = line.split(",")
        rows.append({
            "time_s": float(t), "energy_J": float(energy), "generated_J": float(generated),
            "load_J": float(load), "repair_J": float(repair), "curtailed_J": float(curtailed),
            "depleted": int(float(depleted)), "terminated": int(float(terminated)),
        })
    return rows


def analytic(t: float, cfg: dict) -> dict:
    pv = cfg["inputs"]["pv_power_W"]
    load = cfg["inputs"]["load_power_W"]
    initial = cfg["inputs"]["initial_energy_J"]
    repair = cfg["inputs"]["initial_repair_energy_J"]
    duration = cfg["inputs"]["initial_repair_duration_s"]
    repair_consumed = repair * min(t, duration) / duration
    generated = pv * t
    consumed = load * t
    return {
        "energy_J": max(0.0, initial + generated - consumed - repair_consumed),
        "generated_J": generated,
        "load_J": consumed,
        "repair_J": repair_consumed,
    }


def parse_canonical() -> dict:
    text = CANONICAL.read_text().splitlines()
    survived = float(text[1].split("\t")[2])
    trace = text[3].split("\t")
    return {
        "survived_s": survived,
        "repair_completion_s": float(trace[0]),
        "generated_J": float(trace[1]) * 3_600_000.0,
        "load_J": float(trace[2]) * 3_600_000.0,
        "repair_J": float(trace[3]) * 3_600_000.0,
        "curtailed_J": float(trace[4]) * 3_600_000.0,
        "energy_before_J": float(trace[5]),
        "energy_after_J": float(trace[6]),
    }


def close(actual: float, expected: float, tol: dict, label: str, problems: list[str]) -> float:
    error = abs(actual - expected)
    limit = max(tol["energy_absolute_J"], tol["energy_relative"] * max(abs(expected), 1.0))
    if error > limit:
        problems.append(f"{label}: actual={actual} expected={expected} error={error}")
    return error


def main() -> int:
    cfg = json.loads(CONFIG.read_text())
    canonical = parse_canonical()
    coarse = parse(run_harness(FMU, cfg["time_grid"]["coarse_step_s"], "fmu-coarse-3600"))
    fine = parse(run_harness(FMU, cfg["time_grid"]["fine_step_s"], "fmu-fine-900"))
    tampered = parse(run_harness(TAMPERED, cfg["time_grid"]["coarse_step_s"], "fmu-tampered-3600"))
    fmpy_csv = HERE / "results" / "fmpy-3600.csv"
    fmpy_rows = parse(fmpy_csv) if fmpy_csv.is_file() else None

    problems: list[str] = []
    tol = cfg["tolerances"]
    # 1) FMU vs independent analytic reference (uniform repair draw).
    analytic_errors = []
    for rows, label in ((coarse, "coarse"), (fine, "fine")):
        for row in rows:
            ref = analytic(row["time_s"], cfg)
            for key in ("energy_J", "generated_J", "load_J", "repair_J"):
                analytic_errors.append(close(row[key], ref[key], tol, f"analytic {label} {key} t={row['time_s']}", problems))
            conservation = (cfg["inputs"]["initial_energy_J"] + row["generated_J"] - row["load_J"]
                            - row["repair_J"] - row["curtailed_J"] - row["energy_J"])
            if abs(conservation) > tol["conservation_absolute_J"]:
                problems.append(f"conservation {conservation} at {row['time_s']}")
    # 2) FMU vs canonical at exported boundaries only.
    boundary_problems: list[str] = []
    boundary_errors: list[float] = []
    start = coarse[0]
    boundary_errors.append(close(start["energy_J"], cfg["inputs"]["initial_energy_J"], tol, "canonical t=0 energy", boundary_problems))
    at_repair = next((row for row in coarse if abs(row["time_s"] - canonical["repair_completion_s"]) < 1e-9), None)
    if at_repair is None:
        boundary_problems.append("canonical repair boundary missing in FMU output")
    else:
        for key, expected_key in (("generated_J", "generated_J"), ("load_J", "load_J"),
                                  ("repair_J", "repair_J"), ("energy_J", "energy_after_J")):
            boundary_errors.append(close(at_repair[key], canonical[expected_key], tol,
                                         f"canonical repair boundary {key}", boundary_problems))
    terminal = coarse[-1]
    boundary_errors.append(close(terminal["time_s"], canonical["survived_s"], tol, "canonical depletion time", boundary_problems))
    boundary_errors.append(close(terminal["energy_J"], 0.0, tol, "canonical terminal energy", boundary_problems))
    if abs(terminal["time_s"] - cfg["reference"]["expected_depletion_s"]) > tol["time_absolute_s"]:
        boundary_problems.append("terminal time differs from registered reference")
    problems.extend(boundary_problems)
    # 3) C harness vs fmpy importer.
    fmpy_problems: list[str] = []
    fmpy_max_error = None
    if fmpy_rows is not None:
        if len(fmpy_rows) != len(coarse):
            fmpy_problems.append(f"fmpy row count {len(fmpy_rows)} != {len(coarse)}")
        else:
            fmpy_max_error = 0.0
            for left, right in zip(coarse, fmpy_rows):
                for key in ("time_s", "energy_J", "generated_J", "load_J", "repair_J", "curtailed_J"):
                    fmpy_max_error = max(fmpy_max_error, abs(left[key] - right[key]))
            if fmpy_max_error > tol["energy_absolute_J"]:
                fmpy_problems.append(f"fmpy/harness max error {fmpy_max_error}")
    else:
        fmpy_problems.append("fmpy result missing")
    problems.extend(fmpy_problems)
    # 4) Step sensitivity at common coarse-grid times.
    fine_by_time = {round(row["time_s"], 6): row for row in fine}
    step_problems = []
    for row in coarse:
        key = round(row["time_s"], 6)
        if key in fine_by_time and abs(row["energy_J"] - fine_by_time[key]["energy_J"]) > tol["energy_absolute_J"]:
            step_problems.append(f"step sensitivity at {key}")
            break
    problems.extend(step_problems)
    # 5) Negative test.
    tampered_errors = [abs(row["energy_J"] - analytic(row["time_s"], cfg)["energy_J"]) for row in tampered]
    tampered_error = max(tampered_errors)
    detected = tampered_error > max(tol["energy_absolute_J"], tol["energy_relative"] * 1e6)
    if not detected:
        problems.append("tampered FMU not detected")
    # 6) Known model-assumption difference: uniform repair vs completion settlement.
    # Canonical does not export interior repair-window states, so no numeric claim is made.
    repair_assumption = {
        "status": "MODEL_ASSUMPTION_DIFFERENCE_NOT_COMPARED_AS_TRAJECTORY",
        "fmu": "repair energy spread uniformly over 0..10800 s",
        "canonical": "repair energy committed at repair completion; interior repair states not exported",
        "action": "comparison restricted to t=0, repair completion boundary and depletion; no threshold inflation",
    }
    report = {
        "schema": "poie.s5v-phase2-result/2",
        "config": cfg,
        "fm": {
            "fmu": str(FMU.relative_to(ROOT)), "fmu_sha256": sha(FMU),
            "tampered_fmu": str(TAMPERED.relative_to(ROOT)), "tampered_fmu_sha256": sha(TAMPERED),
            "model_source_sha256": sha(ROOT / "external/fmu-pv-battery/src/model.c"),
            "stubs_source_sha256": sha(ROOT / "external/fmu-pv-battery/src/stubs.c"),
            "model_description_sha256": sha(ROOT / "external/fmu-pv-battery/modelDescription.xml"),
            "harness_source_sha256": sha(ROOT / "external/fmu-pv-battery/src/harness.c"),
            "fmi_version": "3.0",
            "packaging": "FMI 3.0 platform tuple directories with modelIdentifier filename",
        },
        "canonical": {
            "reference": "audit/phase2/canonical-reference.txt", "reference_sha256": sha(CANONICAL),
            "runner": "poie-s5v-energy-reference", "survived_s": canonical["survived_s"],
            "repair_completion_s": canonical["repair_completion_s"],
        },
        "runs": {
            "coarse": {"step_s": cfg["time_grid"]["coarse_step_s"], "rows": len(coarse),
                        "analytic_max_abs_error_J": max(analytic_errors), "canonical_boundary_max_abs_error_J": max(boundary_errors),
                        "terminal": coarse[-1]},
            "fine": {"step_s": cfg["time_grid"]["fine_step_s"], "rows": len(fine), "terminal": fine[-1]},
            "fmpy": {"available": fmpy_rows is not None, "rows": len(fmpy_rows) if fmpy_rows else 0,
                      "max_abs_error_vs_harness_J": fmpy_max_error, "log": "audit/phase2/logs/fmpy.log"},
            "tampered": {"max_abs_error_J": tampered_error, "detected": detected},
        },
        "repair_settlement_model_difference": repair_assumption,
        "problems": problems,
        "gate2": "PASS_SCOPED_EXECUTION_AND_ENDPOINTS" if not problems else "FAIL",
        "trajectory_level_cross_model": "NOT_ESTABLISHED",
    }
    (HERE / "result.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({
        "gate2": report["gate2"], "problems": problems,
        "analytic_max_error": report["runs"]["coarse"]["analytic_max_abs_error_J"],
        "canonical_boundary_max_error": report["runs"]["coarse"]["canonical_boundary_max_abs_error_J"],
        "fmpy_max_error": fmpy_max_error, "tampered_detected": detected,
    }, indent=2))
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main())

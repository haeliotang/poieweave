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

import argparse
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
CANONICAL_TRAJECTORY = HERE / "canonical-trajectory.csv"


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


def _platform(log: Path) -> str | None:
    if not log.is_file():
        return None
    for line in log.read_text().splitlines():
        if line.startswith("platform_tuple="):
            return line.split("=", 1)[1]
    return None


def parse_canonical(expected_sha256: str) -> dict:
    actual = sha(CANONICAL)
    if actual != expected_sha256:
        raise SystemExit(f"canonical reference hash mismatch: actual={actual} expected={expected_sha256}")
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


def parse_canonical_trajectory(expected_sha256: str) -> dict[float, dict]:
    actual = sha(CANONICAL_TRAJECTORY)
    if actual != expected_sha256:
        raise SystemExit(f"canonical trajectory hash mismatch: actual={actual} expected={expected_sha256}")
    result = {}
    for line in CANONICAL_TRAJECTORY.read_text().strip().splitlines()[1:]:
        fields = line.split("\t")
        result[float(fields[0])] = {
            "energy_J": float(fields[1]) * 3_600_000.0,
            "generated_J": float(fields[2]) * 3_600_000.0,
            "load_J": float(fields[3]) * 3_600_000.0,
            "repair_J": float(fields[4]) * 3_600_000.0,
            "repair_active": fields[6] == "true",
            "repair_detail": None if fields[7] == "NOT_EXPORTED" else fields[7],
        }
    return result


def close(actual: float, expected: float, tol: dict, label: str, problems: list[str]) -> float:
    error = abs(actual - expected)
    limit = max(tol["energy_absolute_J"], tol["energy_relative"] * max(abs(expected), 1.0))
    if error > limit:
        problems.append(f"{label}: actual={actual} expected={expected} error={error}")
    return error


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmpy-csv", default=str(HERE / "results" / "fmpy-3600.csv"))
    parser.add_argument("--fmpy-log", default=str(HERE / "logs" / "fmpy.log"))
    args = parser.parse_args()
    cfg = json.loads(CONFIG.read_text())
    canonical = parse_canonical(cfg["canonical_reference"]["sha256"])
    trajectory = parse_canonical_trajectory(cfg["canonical_trajectory"]["sha256"])
    coarse = parse(run_harness(FMU, cfg["time_grid"]["coarse_step_s"], "fmu-coarse-3600"))
    fine = parse(run_harness(FMU, cfg["time_grid"]["fine_step_s"], "fmu-fine-900"))
    tampered = parse(run_harness(TAMPERED, cfg["time_grid"]["coarse_step_s"], "fmu-tampered-3600"))
    fmpy_csv = Path(args.fmpy_csv)
    fmpy_log = Path(args.fmpy_log)
    if not fmpy_csv.is_file():
        raise SystemExit(f"fmpy CSV missing: {fmpy_csv}")
    fmpy_rows = parse(fmpy_csv)

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
    # 2) Canonical checks split by evidence authority.
    #    - repair completion and depletion time are canonical-exported
    #    - t=0 energy is the configured initial condition, not a canonical export
    #    - terminal energy 0 is a constraint, not a canonical export
    boundary_problems: list[str] = []
    boundary_errors: list[float] = []
    initial_problems: list[str] = []
    initial_errors: list[float] = []
    terminal_problems: list[str] = []
    terminal_errors: list[float] = []
    start = coarse[0]
    initial_errors.append(close(start["energy_J"], cfg["inputs"]["initial_energy_J"], tol, "initial-condition energy", initial_problems))
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
    terminal_errors.append(close(terminal["energy_J"], 0.0, tol, "terminal-energy constraint", terminal_problems))
    if abs(terminal["time_s"] - cfg["reference"]["expected_depletion_s"]) > tol["time_absolute_s"]:
        boundary_problems.append("terminal time differs from registered reference")
    problems.extend(boundary_problems + initial_problems + terminal_problems)
    # 3) C harness vs fmpy importer.
    fmpy_problems: list[str] = []
    fmpy_max_error = 0.0
    if len(fmpy_rows) != len(coarse):
        fmpy_problems.append(f"fmpy row count {len(fmpy_rows)} != {len(coarse)}")
    else:
        for left, right in zip(coarse, fmpy_rows):
            for key in ("time_s", "energy_J", "generated_J", "load_J", "repair_J", "curtailed_J"):
                fmpy_max_error = max(fmpy_max_error, abs(left[key] - right[key]))
        if fmpy_max_error > tol["energy_absolute_J"]:
            fmpy_problems.append(f"fmpy/harness max error {fmpy_max_error}")
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
    # 6) Trajectory-level sample comparison. The only expected differences are
    #    caused by the registered repair-settlement assumption, not numerical error.
    trajectory_rows = []
    assumption_errors = []
    coarse_by_time = {round(row["time_s"], 6): row for row in coarse}
    repair_energy = cfg["inputs"]["initial_repair_energy_J"]
    repair_duration = cfg["inputs"]["initial_repair_duration_s"]
    for at_s, obs in sorted(trajectory.items()):
        row = coarse_by_time.get(round(at_s, 6))
        if row is None:
            problems.append(f"no FMU sample at canonical trajectory time {at_s}")
            continue
        delta = row["energy_J"] - obs["energy_J"]
        if at_s < repair_duration:
            expected_delta = -repair_energy * at_s / repair_duration
        else:
            expected_delta = 0.0
        assumption_error = abs(delta - expected_delta)
        assumption_errors.append(assumption_error)
        if assumption_error > max(tol["energy_absolute_J"], 1e-9 * max(abs(expected_delta), 1.0)):
            problems.append(f"trajectory delta not explained at {at_s}: delta={delta} expected={expected_delta}")
        trajectory_rows.append({
            "at_s": at_s,
            "fmu_energy_J": row["energy_J"],
            "canonical_energy_J": obs["energy_J"],
            "delta_J": delta,
            "expected_model_assumption_delta_J": expected_delta,
            "assumption_error_J": assumption_error,
            "fmu_repair_active": None,
            "canonical_repair_active": obs["repair_active"],
            "canonical_repair_detail": obs["repair_detail"],
        })
    # 7) Known model-assumption difference: uniform repair vs completion settlement.
    # Canonical does not export interior repair-window states, so no numeric claim is made.
    repair_assumption = {
        "status": "MODEL_ASSUMPTION_DIFFERENCE_QUANTIFIED",
        "fmu": "repair energy spread uniformly over 0..10800 s",
        "canonical": "repair energy committed at completion; interior samples obtained via neutral zero-quantity probes and trace-derived repair state",
        "expected_delta": "FMU minus canonical = -repair_energy * t / repair_duration for 0 < t < repair_duration; 0 afterwards",
        "action": "observed deltas checked against this pre-registered expression; no threshold inflation",
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
            "runner": cfg["canonical_reference"]["runner"],
            "source_commit": cfg["canonical_reference"]["source_commit"],
            "survived_s": canonical["survived_s"],
            "repair_completion_s": canonical["repair_completion_s"],
            "exported_fields": ["repair completion time", "repair completion generated/load/repair energy",
                                 "repair completion energy before/after", "depletion time"],
            "not_exported": ["repair-window interior states", "t=0 as a runtime record", "terminal energy as a runtime record"],
        },
        "runs": {
            "coarse": {"step_s": cfg["time_grid"]["coarse_step_s"], "rows": len(coarse),
                        "analytic_max_abs_error_J": max(analytic_errors),
                    "canonical_exported_max_abs_error_J": max(boundary_errors),
                    "initial_condition_max_abs_error_J": max(initial_errors),
                    "terminal_constraint_max_abs_error_J": max(terminal_errors),
                    "terminal": coarse[-1]},
            "fine": {"step_s": cfg["time_grid"]["fine_step_s"], "rows": len(fine), "terminal": fine[-1]},
            "fmpy": {"available": True, "rows": len(fmpy_rows),
                      "max_abs_error_vs_harness_J": fmpy_max_error,
                      "csv": str(fmpy_csv), "csv_sha256": sha(fmpy_csv),
                      "log": str(fmpy_log), "log_sha256": sha(fmpy_log) if fmpy_log.is_file() else None},
            "tampered": {"max_abs_error_J": tampered_error, "detected": detected},
        },
        "platform": _platform(fmpy_log),
        "trajectory": {
            "canonical_trajectory": str(CANONICAL_TRAJECTORY.relative_to(ROOT)),
            "canonical_trajectory_sha256": sha(CANONICAL_TRAJECTORY),
            "runner": cfg["canonical_trajectory"]["runner"],
            "rows": trajectory_rows,
            "max_assumption_error_J": max(assumption_errors) if assumption_errors else None,
            "status": "DIFFERENCES_EXPLAINED_BY_REGISTERED_MODEL_ASSUMPTION",
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
        "canonical_exported_max_error": report["runs"]["coarse"]["canonical_exported_max_abs_error_J"],
        "initial_condition_max_error": report["runs"]["coarse"]["initial_condition_max_abs_error_J"],
        "terminal_constraint_max_error": report["runs"]["coarse"]["terminal_constraint_max_abs_error_J"],
        "fmpy_max_error": fmpy_max_error, "tampered_detected": detected,
    }, indent=2))
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main())

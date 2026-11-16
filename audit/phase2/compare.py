#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run the external FMI 3.0 challenge and compare it with canonical energy semantics."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
FMU = ROOT / "external/fmu-pv-battery/poie-pv-battery.fmu"
TAMPERED = ROOT / "external/fmu-pv-battery/poie-pv-battery-tampered.fmu"
HARNESS = ROOT / "external/fmu-pv-battery/build/run_fmu"
CONFIG = HERE / "config.json"


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
    lines = csv.read_text().strip().splitlines()[1:]
    rows = []
    for line in lines:
        t, energy, generated, load, repair, curtailed, depleted, terminated = line.split(",")
        rows.append({
            "time_s": float(t),
            "energy_J": float(energy),
            "generated_J": float(generated),
            "load_J": float(load),
            "repair_J": float(repair),
            "curtailed_J": float(curtailed),
            "depleted": int(float(depleted)),
            "terminated": int(terminated),
        })
    return rows


def reference(t: float, cfg: dict) -> dict:
    pv = cfg["inputs"]["pv_power_W"]
    load = cfg["inputs"]["load_power_W"]
    initial = cfg["inputs"]["initial_energy_J"]
    repair = cfg["inputs"]["initial_repair_energy_J"]
    duration = cfg["inputs"]["initial_repair_duration_s"]
    repair_consumed = repair * min(t, duration) / duration
    generated = pv * t
    consumed = load * t
    energy = initial + generated - consumed - repair_consumed
    return {
        "energy_J": max(0.0, energy),
        "generated_J": generated,
        "load_J": consumed,
        "repair_J": repair_consumed,
    }


def close(actual: float, expected: float, cfg: dict, label: str, problems: list[str]) -> None:
    tol = cfg["tolerances"]
    if abs(actual - expected) > max(tol["energy_absolute_J"], tol["energy_relative"] * max(abs(expected), 1.0)):
        problems.append(f"{label}: actual={actual} expected={expected}")


def compare_run(rows: list[dict], cfg: dict, label: str) -> tuple[list[str], float]:
    problems: list[str] = []
    max_error = 0.0
    for row in rows:
        ref = reference(row["time_s"], cfg)
        for key in ("energy_J", "generated_J", "load_J", "repair_J"):
            error = abs(row[key] - ref[key])
            max_error = max(max_error, error)
            close(row[key], ref[key], cfg, f"{label} {key} t={row['time_s']}", problems)
        if row["energy_J"] > cfg["inputs"]["battery_capacity_J"] + 1e-3:
            problems.append(f"{label} capacity violation")
        conservation = (
            cfg["inputs"]["initial_energy_J"] + row["generated_J"] - row["load_J"] - row["repair_J"]
            - row["curtailed_J"] - row["energy_J"]
        )
        if abs(conservation) > cfg["tolerances"]["conservation_absolute_J"]:
            problems.append(f"{label} conservation {conservation} at {row['time_s']}")
    return problems, max_error


def main() -> int:
    cfg = json.loads(CONFIG.read_text())
    coarse = parse(run_harness(FMU, cfg["time_grid"]["coarse_step_s"], "fmu-coarse-3600"))
    fine = parse(run_harness(FMU, cfg["time_grid"]["fine_step_s"], "fmu-fine-900"))
    tampered = parse(run_harness(TAMPERED, cfg["time_grid"]["coarse_step_s"], "fmu-tampered-3600"))
    problems, coarse_error = compare_run(coarse, cfg, "coarse")
    fine_problems, fine_error = compare_run(fine, cfg, "fine")
    problems.extend(fine_problems)
    tolerance = cfg["tolerances"]
    coarse_end = coarse[-1]
    fine_end = fine[-1]
    for label, end in (("coarse", coarse_end), ("fine", fine_end)):
        if abs(end["time_s"] - cfg["reference"]["expected_depletion_s"]) > tolerance["time_absolute_s"]:
            problems.append(f"{label} depletion time {end['time_s']}")
        if not end["depleted"] or not end["terminated"]:
            problems.append(f"{label} not reported depleted/terminated")
        if abs(end["generated_J"] - cfg["reference"]["expected_generated_J"]) > tolerance["energy_absolute_J"]:
            problems.append(f"{label} terminal generated")
        if abs(end["load_J"] - cfg["reference"]["expected_load_J"]) > tolerance["energy_absolute_J"]:
            problems.append(f"{label} terminal load")
        if abs(end["repair_J"] - cfg["reference"]["expected_repair_J"]) > tolerance["energy_absolute_J"]:
            problems.append(f"{label} terminal repair")
    # Step sensitivity at common coarse-grid times.
    fine_by_time = {round(row["time_s"], 6): row for row in fine}
    for row in coarse:
        key = round(row["time_s"], 6)
        if key in fine_by_time and abs(row["energy_J"] - fine_by_time[key]["energy_J"]) > tolerance["energy_absolute_J"]:
            problems.append(f"step sensitivity at {key}")
            break
    # Negative test: the tampered PV model must exceed tolerance.
    tampered_problems, tampered_error = compare_run(tampered, cfg, "tampered")
    detected = tampered_error > max(tolerance["energy_absolute_J"], tolerance["energy_relative"] * 1e6)
    if not detected:
        problems.append("tampered FMU not detected")
    canonical = (HERE / "canonical-reference.txt").read_text()
    if "blocked:energy_depleted" not in canonical or "79.000000" not in canonical:
        problems.append("canonical reference terminal")
    if "10800.000\t7.500000\t9.000000" not in canonical:
        problems.append("canonical reference 3 h ledger")
    report = {
        "schema": "poie.s5v-phase2-result/1",
        "config": cfg,
        "fm": {
            "fmu": str(FMU.relative_to(ROOT)),
            "fmu_sha256": sha(FMU),
            "tampered_fmu": str(TAMPERED.relative_to(ROOT)),
            "tampered_fmu_sha256": sha(TAMPERED),
            "model_source": "external/fmu-pv-battery/src/model.c",
            "model_source_sha256": sha(ROOT / "external/fmu-pv-battery/src/model.c"),
            "model_description_sha256": sha(ROOT / "external/fmu-pv-battery/modelDescription.xml"),
            "harness_source_sha256": sha(ROOT / "external/fmu-pv-battery/src/harness.c"),
            "fmi_version": "3.0",
        },
        "canonical": {
            "reference": "audit/phase2/canonical-reference.txt",
            "reference_sha256": sha(HERE / "canonical-reference.txt"),
            "runner": "poie-s5v-energy-reference",
            "depletion_s": coarse_end["time_s"],
        },
        "runs": {
            "coarse": {"step_s": cfg["time_grid"]["coarse_step_s"], "rows": len(coarse), "max_abs_error_J": coarse_error,
                         "terminal": coarse_end},
            "fine": {"step_s": cfg["time_grid"]["fine_step_s"], "rows": len(fine), "max_abs_error_J": fine_error,
                     "terminal": fine_end},
            "tampered": {"step_s": cfg["time_grid"]["coarse_step_s"], "max_abs_error_J": tampered_error,
                          "detected": detected},
        },
        "problems": problems,
        "gate2": "PASS" if not problems else "FAIL",
    }
    (HERE / "result.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"gate2": report["gate2"], "problems": problems, "coarse_error": coarse_error,
                      "tampered_error": tampered_error, "depletion_s": coarse_end["time_s"]}, indent=2))
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main())

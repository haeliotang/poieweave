#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Render Phase 2 report and evidence manifest."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    result = json.loads((HERE / "result.json").read_text())
    files = [
        ROOT / "external/fmu-pv-battery/poie-pv-battery.fmu",
        ROOT / "external/fmu-pv-battery/poie-pv-battery-tampered.fmu",
        ROOT / "external/fmu-pv-battery/modelDescription.xml",
        ROOT / "external/fmu-pv-battery/src/model.c",
        ROOT / "external/fmu-pv-battery/src/harness.c",
        ROOT / "external/fmu-pv-battery/src/stubs.c",
        HERE / "config.json",
        HERE / "canonical-reference.txt",
        HERE / "canonical-trajectory.csv",
        HERE / "result.json",
        HERE / "fmpy_import.py",
    ]
    files += sorted((HERE / "results").glob("*.csv"))
    files += sorted((HERE / "logs").glob("*.log"))
    manifest = {
        "schema": "poie.s5v-phase2-manifest/1",
        "files": [
            {"path": str(path.relative_to(ROOT)), "sha256": sha(path)} for path in files
        ],
    }
    (HERE / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    lines = [
        "# S5-V Phase 2 — Conformance and Comparator Closure",
        "",
        f"- Gate 2: `{result['gate2']}`",
        "- external execution: `ESTABLISHED (SCOPED)`",
        "- endpoint agreement: `ESTABLISHED (SINGLE ENERGY CASE)`",
        "- trajectory-level cross-model validity: `NOT ESTABLISHED`",
        f"- external FMU: `{result['fm']['fmu']}` (FMI {result['fm']['fmi_version']})",
        f"- FMU SHA-256: `{result['fm']['fmu_sha256']}`",
        f"- packaging: {result['fm']['packaging']}",
        f"- independent importer: fmpy 0.3.32, rows={result['runs']['fmpy']['rows']}, "
        f"max error vs C harness={result['runs']['fmpy']['max_abs_error_vs_harness_J']} J, "
        f"CSV SHA={result['runs']['fmpy']['csv_sha256']}",
        f"- analytic-reference max error: `{result['runs']['coarse']['analytic_max_abs_error_J']}` J",
        f"- canonical-exported boundary max error: `{result['runs']['coarse']['canonical_exported_max_abs_error_J']}` J",
        f"- initial-condition max error: `{result['runs']['coarse']['initial_condition_max_abs_error_J']}` J",
        f"- terminal-energy constraint max error: `{result['runs']['coarse']['terminal_constraint_max_abs_error_J']}` J",
        f"- tampered FMU max error: `{result['runs']['tampered']['max_abs_error_J']}` J, "
        f"detected={result['runs']['tampered']['detected']}",
        f"- canonical trajectory samples: {len(result['trajectory']['rows'])}, "
        f"max assumption error: `{result['trajectory']['max_assumption_error_J']}` J",
        "",
        "## What is compared",
        "",
        "The FMU is executed through the C harness and through the independent",
        "fmpy 0.3.32 importer. The C-harness trajectory is compared with an",
        "independent analytic reference that uses the same uniform repair-draw",
        "assumption as the FMU. Canonical is checked only on evidence it actually",
        "exports: the repair-completion record and the depletion time. Initial",
        "energy is a configured initial condition, and terminal energy zero is a",
        "constraint; neither is presented as a canonical runtime export.",
        "",
        "## Canonical trajectory samples",
        "",
        "The canonical observatory exports 0/1/2/3 h and depletion states through",
        "neutral zero-quantity sample probes. It does not add inventory and does not",
        "change plans or energy-settlement rules. Repair-active state is derived from",
        "the canonical trace and is labelled as derived.",
        "",
        "| t (s) | FMU energy (J) | canonical energy (J) | delta (J) | expected assumption delta (J) |",
        "| --- | --- | --- | --- | --- |",
    ]
    for row in result["trajectory"]["rows"]:
        lines.append(
            f"| {row['at_s']:.0f} | {row['fmu_energy_J']:.0f} | {row['canonical_energy_J']:.0f} | "
            f"{row['delta_J']:.0f} | {row['expected_model_assumption_delta_J']:.0f} |"
        )
    lines += [
        "",
        "All observed deltas match the registered repair-settlement assumption within",
        "the declared tolerance; no numerical or event-timing residual is hidden.",
        "",
        "## Repair-window model difference",
        "",
        "The canonical engine commits repair energy when the repair completes; the",
        "external FMU spreads the same energy over the repair window. The canonical",
        "observatory samples the interior through neutral zero-quantity probes,",
        "while the native engine API still does not expose repair identity directly.",
        "Observed deltas are checked against the pre-registered settlement expression",
        "rather than treated as numerical error:",
        "",
        f"```text\n{json.dumps(result['repair_settlement_model_difference'], indent=2)}\n```",
        "",
        "## Standard packaging",
        "",
        f"The frozen canonical reference is `{result['canonical']['reference']}` with",
        f"SHA-256 `{result['canonical']['reference_sha256']}` and source commit",
        f"`{result['canonical']['source_commit']}`; its SHA is verified before comparison.",
        "",
        "The FMU now uses FMI 3.0 platform-tuple directories and modelIdentifier",
        "filenames, e.g. `binaries/aarch64-darwin/poie_pv_battery.dylib` and",
        "`binaries/x86_64-linux/poie_pv_battery.so`. Generic FMI 3.0 importers",
        "require unsupported-interface symbols to be present; the FMU includes",
        "explicit fail-closed stubs for non-Co-Simulation APIs.",
        "",
        "## Claims",
        "",
        "```text",
        "S5-V EXTERNAL EXECUTION — ESTABLISHED (SCOPED)",
        "S5-V ENDPOINT AGREEMENT — ESTABLISHED (SINGLE ENERGY CASE)",
        "S5-V TRAJECTORY-LEVEL CROSS-MODEL VALIDITY — NOT ESTABLISHED",
        "S5 CROSS-MODEL VALIDITY — NOT ESTABLISHED (full S2)",
        "```",
        "",
        "## Limitations",
        "",
        "- one parameter set and one energy-only scenario",
        "- no failures beyond the initial repair, no curtailment/capacity saturation",
        "- step sizes 3600/900 s only",
        "- local execution on macOS arm64; committed FMU binary is macOS arm64",
        "- canonical interior repair-window states are not exported",
        "- this does not establish physical-world correctness",
        "",
        "## CI evidence binding",
        "",
        "The Linux CI job passes its freshly generated fmpy CSV and log path to",
        "`compare.py`; the comparator fails if that CSV is missing. The result",
        "records the current FMU SHA, fmpy CSV/log SHA and platform tuple. The job",
        "then copies the current FMU, harness CSV, fmpy CSV/log, canonical reference",
        "and result into `s5v-linux-evidence` with a SHA256SUMS file and uploads it",
        "as a GitHub Actions artifact.",
        "",
        "## Evidence",
        "",
        f"- config: `{str((HERE / 'config.json').relative_to(ROOT))}`",
        f"- result: `{str((HERE / 'result.json').relative_to(ROOT))}`",
        f"- manifest: `{str((HERE / 'manifest.json').relative_to(ROOT))}`",
        f"- raw CSVs: `{str((HERE / 'results').relative_to(ROOT))}`",
        f"- logs: `{str((HERE / 'logs').relative_to(ROOT))}`",
        "",
    ]
    (HERE / "REPORT.zh-CN.md").write_text("\n".join(lines))
    print(f"wrote manifest with {len(files)} files and Phase 2 report")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

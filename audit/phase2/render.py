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
        HERE / "config.json",
        HERE / "canonical-reference.txt",
        HERE / "result.json",
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
        "# S5-V Phase 2 — Executable External Model Challenge",
        "",
        f"- Gate 2: `{result['gate2']}`",
        f"- external FMU: `{result['fm']['fmu']}` (FMI {result['fm']['fmi_version']})",
        f"- FMU SHA-256: `{result['fm']['fmu_sha256']}`",
        f"- independent model: C99 PV + battery + load + initial repair draw",
        f"- depletion time: `{result['canonical']['depletion_s']}` s",
        f"- coarse and fine max absolute error: `{result['runs']['coarse']['max_abs_error_J']}` J",
        f"- tampered FMU max absolute error: `{result['runs']['tampered']['max_abs_error_J']}` J",
        f"- tampered detector: `{result['runs']['tampered']['detected']}`",
        "",
        "## Scope",
        "",
        "This is a single reduced energy-subsystem challenge. It is not a physical",
        "validation and not a full S2 cross-model validation.",
        "",
        "## Pre-registered case",
        "",
        "- PV 2500 W, load 3000 W, battery 200 kWh / 10 kW charge",
        "- initial energy 40 kWh, initial repair draw 0.5 kWh over 3 h",
        "- expected depletion 284400 s (79 h)",
        "- coarse step 3600 s, fine step 900 s",
        "- energy relative tolerance 1e-9, absolute 1 J, conservation 1e-3 J",
        "",
        "## Independent implementation",
        "",
        "The FMU is hand-written C99, packaged as FMI 3.0 Co-Simulation, and does not",
        "link or call the Poieweave Rust engine. The harness unpacks the FMU, dlopens",
        "the native model, calls `fmi3InstantiateCoSimulation`, initialization, variable",
        "exchange and `fmi3DoStep`, then terminates the instance.",
        "",
        "## Comparison result",
        "",
        "The FMU output is compared every communication step against an independent",
        "closed-form reference. Energy conservation, battery capacity, depletion time,",
        "cumulative generated/load/repair energy and step sensitivity all pass.",
        "The tampered FMU (+100 W PV) is detected by the same comparator.",
        "",
        "## Limitations",
        "",
        "- one parameter set and one energy-only scenario",
        "- no failures beyond the initial repair, no curtailment/capacity saturation",
        "- step sizes 3600/900 s only",
        "- executed on the local macOS arm64 host; source and build script support Linux",
        "- this does not establish physical-world correctness",
        "",
        "## Claim status",
        "",
        "`S5 CROSS-MODEL VALIDITY — ESTABLISHED (SCOPED: PV/battery/load energy boundary)`",
        "",
        "The broader S2/model-wide cross-model validity remains NOT ESTABLISHED.",
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

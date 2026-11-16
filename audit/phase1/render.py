#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Render the Phase 1 audit report and claim matrix from report.json."""

from __future__ import annotations

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main() -> int:
    data = json.loads((HERE / "report.json").read_text())
    lines = [
        "# S5-V Phase 1 — Independent Certificate Audit",
        "",
        f"- baseline requested: `{data['baseline_expected']}`",
        f"- baseline actual HEAD: `{data['baseline_actual']}`",
        f"- selected sample: {len(data['selected'])} / 36 certificates",
        f"- all-certificate hash checks: {data['all_certificate_hash_checks']['certificates']} checked, "
        f"{len(data['all_certificate_hash_checks']['failures'])} failures",
        f"- Gate 1: `{data['gate1']}`",
        "",
        "## Method",
        "",
        "The audit harness is independent of the repository certificate generator and verifier:",
        "",
        "- re-executes each fixed-commit runner and compares stdout byte-for-byte with `expected.txt`",
        "- recomputes all certificate SHA-256 bindings with a separate implementation",
        "- parses output rows and checks claim arithmetic/constraints directly",
        "- runs four negative certificate mutations (expected, engine, claim, model)",
        "- records raw runner output under `audit/phase1/evidence/` with SHA-256",
        "",
        "No historical expected output or certificate was modified.",
        "",
        "## Status counts",
        "",
        "```text",
    ]
    for status, count in sorted(data["counts"].items()):
        lines.append(f"{status:<26} {count}")
    lines += ["```", "", "## Claim matrix", "", "| claim | certificate | runner | status | first divergence |", "| --- | --- | --- | --- | --- |"]
    for row in data["results"]:
        cert = row["certificate"].replace("experiments/", "")
        lines.append(
            f"| {row['claim']} | `{cert}` | `{row['runner']}` | {row['status']} | {row['first_divergence']} |"
        )
    lines += ["", "## Limitations", ""]
    for row in data["results"]:
        if row["limitations"]:
            lines.append(f"- **{row['claim']}**: {row['limitations']}")
    lines += [
        "",
        "## Negative tests",
        "",
        "| mutation | rejected |",
        "| --- | --- |",
    ]
    for test in data["negative_tests"]:
        lines.append(f"| {test['mutation']} | {test['rejected']} |")
    lines += [
        "",
        "## Gate 1 ruling",
        "",
        f"`{data['gate1']}`",
        "",
        "The 12 sampled claims have no unexplained mismatch. Three claims are aggregate-only",
        "and therefore `VERIFIED_WITH_LIMITATIONS`: their aggregate statistics reproduce but",
        "the raw random trial/schedule stream is not exported for independent re-derivation.",
        "The other 24 certificates have only independent hash checks in this phase, not",
        "runner re-execution, and remain a residual audit risk.",
        "",
        "Independence here is implementation-level: the harness does not reuse the repository",
        "certificate generator or verifier, but it was executed by the same agent, not by an",
        "independent third-party reviewer.",
        "",
    ]
    (HERE / "REPORT.zh-CN.md").write_text("\n".join(lines))
    matrix = {
        "schema": "poie.s5v-phase1-matrix/1",
        "baseline": data["baseline_actual"],
        "gate1": data["gate1"],
        "rows": [
            {
                "claim": row["claim"],
                "certificate": row["certificate"],
                "experiment": row["experiment"],
                "runner": row["runner"],
                "classification": row["classification"],
                "status": row["status"],
                "limitations": row["limitations"],
                "first_divergence": row["first_divergence"],
                "evidence": row["evidence"],
                "evidence_sha256": row["evidence_sha256"],
            }
            for row in data["results"]
        ],
        "all_certificate_hash_checks": data["all_certificate_hash_checks"],
        "negative_tests": data["negative_tests"],
    }
    (HERE / "matrix.json").write_text(json.dumps(matrix, indent=2, ensure_ascii=False) + "\n")
    print("wrote audit/phase1/REPORT.zh-CN.md and matrix.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

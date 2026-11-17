#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Audit N6 evidence consistency and mutation coverage."""

from __future__ import annotations

import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
CI = ROOT / ".github/workflows/ci.yml"


def main() -> int:
    problems = []
    manifest = json.loads((PHASE / "manifest.json").read_text())
    if manifest["schema"] != "poie.s5v-n6-evidence/1":
        problems.append("manifest schema")
    if manifest["baseline"] != "c38ea8500426a5bc9ea9e4b521a9c4fcc0bf9423":
        problems.append("baseline")
    for name, filename in [("lifetime", "resource-lifetime"), ("scheduling", "scheduling"),
                            ("dependencies", "dependencies")]:
        data = json.loads((PHASE / "differences" / f"{filename}.json").read_text())
        for entry in data["cases"]:
            status = entry.get("status")
            if status in ("UNEXPLAINED", "FAIL"):
                problems.append(f"{name}:{entry.get('case')}: {status}")
    for name in ["lifetime-results", "scheduling-results", "dependencies-results", "contract-results"]:
        data = json.loads((PHASE / "negative-controls" / f"{name}.json").read_text())
        if not data["results"] or not all(entry["detected"] for entry in data["results"]):
            problems.append(f"negative coverage {name}")
    restoration = json.loads((PHASE / "restoration/restoration-state.json").read_text())
    if not all(record["restoration_agreement"] for record in restoration["records"]):
        problems.append("restoration agreement")
    for record in restoration["records"]:
        for target in record["targets"]:
            if target["selected_resources"] is not None or target["selected_resources_observed"] is not False:
                problems.append("restoration selected_resources must be explicitly unobserved")
    if not (PHASE / "EVIDENCE.md").is_file():
        problems.append("evidence document")
    report = (PHASE / "REPORT.zh-CN.md").read_text()
    for token in ["N6-B CONTRACT-DRIVEN SCHEDULING", "S5-V N6 EVIDENCE HARDENING",
                  "N6 CONTRACT-DRIVEN COORDINATION"]:
        if token not in report:
            problems.append(f"report missing {token}")
    for stale in ["本轮仅完成 N6-A", "N6-B..E 保持未完成", "DIFFERENT_BUT_VALID_DECISION"]:
        if stale in report:
            problems.append(f"report stale: {stale}")
    evidence_text = (PHASE / "EVIDENCE.md").read_text()
    for token in ["FROZEN_TRACKED_EVIDENCE", "CURRENT_RUN_EXECUTION_EVIDENCE", "clean-checkout"]:
        if token not in evidence_text:
            problems.append(f"evidence doc missing {token}")
    ci = CI.read_text()
    for token in ["s5v-n6-linux", "audit/phase6/compare_dependencies.py",
                  "audit/phase6/negative_contracts.py", "audit/phase6/execution_manifest.py"]:
        if token not in ci:
            problems.append(f"CI missing {token}")
    v2 = json.loads((ROOT / "docs/research/canonical-claim-certificate-index-v2.json").read_text())
    if len(v2["certificates"]) != 41:
        problems.append("v2 certificate count")
    if not (ROOT / "audit/certificates/v1-frozen.json").is_file():
        problems.append("v1 freeze")
    if problems:
        print("N6-E evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print("ok N6-E evidence integrity and mutation coverage")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

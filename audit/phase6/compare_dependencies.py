#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N6-C/D dependency and event-boundary comparator."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CASES = ["N6-C-tool-1", "N6-C-tool-9000", "N6-C-tool-10800", "N6-C-tool-12000",
         "N6-C-bus-9000", "N6-C-bus-10800", "N6-D-same-10800",
         "N6-D-horizon-completion", "N6-D-horizon-failure"]


def external(case: str) -> dict:
    rows = list(csv.DictReader((PHASE / "external" / f"{case}-fmpy-900.csv").open()))
    last = rows[-1]
    return {key: float(value) for key, value in last.items() if key != "terminated"} | {
        "terminated": int(float(last["terminated"]))}


def compare(case: str) -> dict:
    canonical = json.loads((PHASE / "canonical" / f"{case}-audit.json").read_text())
    ext = external(case)
    failure_s = ext["failureAppliedTime_s"]
    failure_expected = canonical["failure_applied_s"]
    findings = []
    if failure_expected is None:
        if failure_s >= 0:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "reason": "failure processed at horizon"})
    elif abs(failure_s - failure_expected) > 1e-6:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT"})
    interrupted = bool(ext["interrupted"])
    if canonical["repair_interrupted"] != interrupted:
        kind = "UPSTREAM_DEPENDENCY_MISMATCH" if canonical["upstream_required"] else "EXECUTION_DEPENDENCY_MISMATCH"
        findings.append({"class": kind})
    restored = bool(ext["restored1"]) if canonical["tasks"] >= 1 else False
    if canonical["restored_targets"] and not restored:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH"})
    if not canonical["restored_targets"] and restored:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "restored after interruption"})
    # tool state and allocation must be independently observable
    if case.startswith("N6-C-tool"):
        if ext["toolTaskMask"] < 1:
            findings.append({"class": "REQUIRED_TOOL_MISSING", "reason": "tool not assigned"})
        if ext["toolAcquired_s"] != 0.0:
            findings.append({"class": "RESOURCE_CONTRACT_VIOLATION", "reason": "tool acquired late"})
        if failure_s >= 0 and ext["toolAvailable"] != 0:
            findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH", "reason": "tool remained available"})
        if not interrupted:
            if abs(ext["toolReleased_s"] - ext["makespan_s"]) > 1e-6:
                findings.append({"class": "RESOURCE_LIFETIME_MISMATCH", "reason": "tool released before group commit"})
        elif abs(ext["toolReleased_s"] - failure_s) > 1e-6:
            findings.append({"class": "RESOURCE_LIFETIME_MISMATCH", "reason": "tool release at interruption"})
    # horizon: no failure event may be processed at horizon
    if "horizon" in case and failure_s >= 0:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT", "reason": "horizon accepted failure"})
    # event order at makespan: completion before same-time failure
    order = None
    if case == "N6-D-same-10800":
        if abs(failure_s - ext["makespan_s"]) <= 1e-6 and ext["completedCount"] > 0 and not interrupted:
            order = "COMPLETION_BEFORE_FAILURE"
        else:
            findings.append({"class": "EVENT_ORDER_MISMATCH"})
    result = {
        "schema": "poie.s5v-n6cd-boundary/1",
        "case": case,
        "canonical": canonical,
        "external": ext,
        "failure_expected_s": failure_expected,
        "failure_external_s": failure_s,
        "event_order": order,
        "findings": findings,
        "first_divergence": findings[0] if findings else {"class": "NO_DIVERGENCE", "at_s": None},
        "status": "EXPLAINED" if not findings else "UNEXPLAINED",
    }
    return result


def main() -> int:
    results = [compare(case) for case in CASES]
    (PHASE / "differences" / "dependencies.json").write_text(
        json.dumps({"schema": "poie.s5v-n6cd-boundary-set/1", "cases": results}, indent=2) + "\n"
    )
    for result in results:
        print(result["case"], result["status"], result["findings"])
    return 0 if all(result["status"] == "EXPLAINED" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

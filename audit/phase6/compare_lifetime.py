#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N6-A group-level resource lifetime comparator."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent


def read_csv(path: Path) -> list[dict]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle))


def compare(failure_s: float) -> dict:
    canonical = json.loads((PHASE / "canonical" / f"N6-A-{failure_s:.0f}-audit.json").read_text())
    rows = read_csv(PHASE / "external" / f"N6-A-{failure_s:.0f}-fmpy-900.csv")
    last = rows[-1]
    external = {
        "makespan_s": float(last["makespan_s"]),
        "completed": int(float(last["completedCount"])),
        "interrupted": int(float(last["interrupted"])),
        "failure_applied_s": float(last["failureAppliedTime_s"]),
        "group_committed_s": float(last["makespan_s"]) if int(float(last["interrupted"])) == 0 else None,
        "restored": bool(float(last["pumpRestored"])) and bool(float(last["latheRestored"])),
        "terminal_s": float(last["time_s"]),
        "t1_end_s": float(last["t1End_s"]),
        "t2_end_s": float(last["t2End_s"]),
    }
    findings = []
    if abs(external["failure_applied_s"] - failure_s) > 1e-6:
        findings.append({"class": "EVENT_TIME_DISPLACEMENT"})
    # Group-level contract: selected resource held until group commit; failure
    # before makespan interrupts, failure at makespan is after commit.
    expected_interrupt = failure_s < canonical["makespan_s"] - 1e-9
    if external["interrupted"] != int(expected_interrupt):
        findings.append({"class": "RESOURCE_LIFETIME_MISMATCH",
                         "reason": "group-level hold mismatch", "expected_interrupted": expected_interrupt,
                         "external_interrupted": bool(external["interrupted"])})
    if canonical["repair_interrupted"] != bool(external["interrupted"]):
        findings.append({"class": "EXECUTION_DEPENDENCY_MISMATCH"})
    if canonical["restoration_committed"] != external["restored"]:
        findings.append({"class": "RESTORATION_COMMIT_MISMATCH"})
    if not expected_interrupt:
        if external["group_committed_s"] is None:
            findings.append({"class": "RESTORATION_COMMIT_MISMATCH", "reason": "missing group commit"})
        elif abs(external["group_committed_s"] - canonical["group_committed_s"]) > 1e-6:
            findings.append({"class": "EVENT_TIME_DISPLACEMENT", "kind": "group_commit"})
    if external["t1_end_s"] < 7200.0 - 1e-6:
        findings.append({"class": "RESOURCE_LIFETIME_MISMATCH", "reason": "lathe step completion moved"})
    result = {
        "schema": "poie.s5v-n6a-lifetime/1",
        "case": f"N6-A-{failure_s:.0f}",
        "failure_s": failure_s,
        "contract": {
            "resource_release_semantics": "GROUP_LEVEL",
            "step_completed_lathe_s": 7200.0,
            "step_completed_pump_s": 10800.0,
            "group_commit_s": canonical["makespan_s"],
            "step_level_release": "NOT_REPRESENTABLE",
        },
        "canonical": canonical,
        "external": external,
        "findings": findings,
        "first_divergence": findings[0] if findings else {"class": "NO_DIVERGENCE", "at_s": None},
        "status": "EXPLAINED" if not findings else "UNEXPLAINED",
    }
    return result


def main() -> int:
    results = [compare(failure) for failure in [7199.0, 7200.0, 7201.0, 9000.0, 10800.0]]
    (PHASE / "differences" / "resource-lifetime.json").write_text(
        json.dumps({"schema": "poie.s5v-n6a-lifetime-set/1", "cases": results}, indent=2) + "\n"
    )
    for result in results:
        print(result["case"], result["status"], result["findings"])
    return 0 if all(result["status"] == "EXPLAINED" for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())

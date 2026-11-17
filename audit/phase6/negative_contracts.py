#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Mutation controls for N6 contract validation, not case-name classification."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


sched = load("n6_sched", PHASE / "compare_scheduling.py")
deps = load("n6_deps", PHASE / "compare_dependencies.py")


def run_with(paths: list[Path], replacement: dict[Path, str], action) -> dict:
    backups = {path: path.read_text() for path in paths}
    try:
        for path, text in replacement.items():
            path.write_text(text)
        return action()
    finally:
        for path, text in backups.items():
            path.write_text(text)


def mutate_contract(case: str, old: str, new: str) -> str:
    path = PHASE / "contracts" / f"{case}.contract"
    text = path.read_text()
    if old not in text:
        raise SystemExit(f"contract anchor missing: {case} {old}")
    return text.replace(old, new, 1)


results = []


def record(name: str, expected: str, classes: set[str], detected: bool) -> None:
    results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes)})
    print(name, "PASS" if detected else "FAIL", sorted(classes))


# 1. Task 2 mask allows only robot-m1; actual schedule uses robot-m2.
contract = PHASE / "contracts/N6-B00.contract"
text = mutate_contract("N6-B00", "t2RobotMask=3", "t2RobotMask=1")
result = run_with([contract], {contract: text}, lambda: sched.compare("N6-B00"))
classes = {finding["class"] for finding in result["findings"]}
record("N6-B00-mask-forbidden-robot", "RESOURCE_ELIGIBILITY_VIOLATION", classes,
       "RESOURCE_ELIGIBILITY_VIOLATION" in classes)

# 2. No eligible robot exists.
text = mutate_contract("N6-B00", "t1RobotMask=3", "t1RobotMask=0")
result = run_with([contract], {contract: text}, lambda: sched.compare("N6-B00"))
classes = {finding["class"] for finding in result["findings"]}
record("N6-B00-no-eligible-robot", "RESOURCE_ELIGIBILITY_VIOLATION", classes,
       "RESOURCE_ELIGIBILITY_VIOLATION" in classes)

# 3. Same priority input but a priority-2 task starts before priority-1 on a shared robot.
schedule = PHASE / "canonical/N6-B08-schedule.csv"
lines = schedule.read_text().splitlines()
# current canonical B08 priorities are 1,2; make them 2,1 without changing order
lines[1] = lines[1].replace("\t1\t", "\t2\t", 1)
lines[2] = lines[2].replace("\t2\t", "\t1\t", 1)
result = run_with([schedule], {schedule: "\n".join(lines) + "\n"}, lambda: sched.compare("N6-B08"))
classes = {finding["class"] for finding in result["findings"]}
record("N6-B08-same-priority-order-violation", "POLICY_CONTRACT_VIOLATION", classes,
       "POLICY_CONTRACT_VIOLATION" in classes)

# 4. Required tool is missing from the external schedule.
tool_csv = PHASE / "external/N6-C-tool-9000-fmpy-900.csv"
lines = tool_csv.read_text().splitlines()
header = lines[0].split(",")
row = lines[-1].split(",")
mask_index, acquired_index, released_index = header.index("toolTaskMask"), header.index("toolAcquired_s"), header.index("toolReleased_s")
row[mask_index] = "0.000000000"
row[acquired_index] = "-1.000000000"
row[released_index] = "-1.000000000"
result = run_with([tool_csv], {tool_csv: "\n".join([lines[0], ",".join(row)]) + "\n"},
                  lambda: deps.compare("N6-C-tool-9000"))
classes = {finding["class"] for finding in result["findings"]}
record("N6-C-tool-9000-missing-tool", "REQUIRED_TOOL_MISSING", classes, "REQUIRED_TOOL_MISSING" in classes)

# 5. Tool released before group commit.
row = lines[-1].split(",")
row[released_index] = "8999.000000000"
result = run_with([tool_csv], {tool_csv: "\n".join([lines[0], ",".join(row)]) + "\n"},
                  lambda: deps.compare("N6-C-tool-9000"))
classes = {finding["class"] for finding in result["findings"]}
record("N6-C-tool-9000-early-tool-release", "RESOURCE_LIFETIME_MISMATCH", classes,
       "RESOURCE_LIFETIME_MISMATCH" in classes)

# 6. Contract input changes while the old schedule is kept.
text = mutate_contract("N6-B00", "t1Duration_s=7200", "t1Duration_s=9999")
result = run_with([contract], {contract: text}, lambda: sched.compare("N6-B00"))
classes = {finding["class"] for finding in result["findings"]}
record("N6-B00-contract-duration-mismatch", "RESOURCE_CONTRACT_VIOLATION", classes,
       "RESOURCE_CONTRACT_VIOLATION" in classes)

# 7. Frozen evidence mutation is rejected by check.py.
evidence = PHASE / "canonical/N6-B00-summary.json"
original = evidence.read_text()
bad = original.replace("\"makespan_s\": 10800.0", "\"makespan_s\": 1.0", 1)
try:
    evidence.write_text(bad)
    check = subprocess.run([sys.executable, str(PHASE / "check.py")], capture_output=True, text=True)
    detected = check.returncode != 0
finally:
    evidence.write_text(original)
record("N6-B00-frozen-evidence-mutation", "EVIDENCE_INTEGRITY_FAILURE", {"EVIDENCE_INTEGRITY_FAILURE"}, detected)

output = PHASE / "negative-controls/contract-results.json"
output.write_text(json.dumps({"schema": "poie.s5v-n6-contract-negative-controls/2", "results": results}, indent=2) + "\n")
raise SystemExit(0 if all(entry["detected"] for entry in results) else 1)

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent verifier for exported S1 plans.

Reads model TOML and a JSON plan export. For every RESTORABLE case it replays
the plan's steps against a fresh inventory and checks that each process input is
available, the final target is restored, and the duration does not exceed the
declared horizon. It does not call the Rust closure or planner.
"""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QUANTITY = re.compile(r"^\s*([0-9]+(?:\.[0-9]+)?)\s*([A-Za-z/ ]+?)\s*$")
UNITS = {
    "kg": 1.0,
    "g": 0.001,
    "item": 1.0,
    "items": 1.0,
    "kwh": 3_600_000.0,
    "wh": 3_600.0,
    "h": 3600.0,
    "min": 60.0,
    "s": 1.0,
    "l": 1.0,
    "liter": 1.0,
}


def si(text: str) -> float:
    match = QUANTITY.match(text)
    if not match:
        raise ValueError(f"cannot parse quantity {text!r}")
    value, unit = float(match.group(1)), match.group(2).strip().lower()
    if unit not in UNITS:
        raise ValueError(f"unknown unit {unit!r}")
    return value * UNITS[unit]


def inventory(model: dict) -> dict[str, float]:
    result: dict[str, float] = {}
    for lot in model.get("lot", []):
        if lot.get("quality", "accepted") == "accepted":
            result[lot["resource"]] = result.get(lot["resource"], 0.0) + si(lot["quantity"])
    return result


def verify_plan(case: dict) -> str | None:
    if case["outcome"] != "RESTORABLE":
        return None
    model_path = ROOT / case["model"]
    with model_path.open("rb") as handle:
        model = tomllib.load(handle)
    processes = {process["id"]: process for process in model.get("process", [])}
    stock = inventory(model)
    restored: set[str] = set()
    duration = 0.0
    for step in case["steps"]:
        process = processes.get(step["process"])
        if process is None:
            return f"unknown process {step['process']}"
        for item in process.get("inputs", []):
            resource, need = item["resource"], si(item["quantity"])
            have = stock.get(resource, 0.0)
            if have + 1e-9 < need:
                return f"step {step['process']}: missing {resource}: need {need}, have {have}"
            stock[resource] = have - need
        for item in process.get("outputs", []):
            stock[item["resource"]] = stock.get(item["resource"], 0.0) + si(item["quantity"])
        for asset in process.get("restores", []):
            restored.add(asset)
        duration += step["duration_s"]
    target = case["target"]
    if target not in restored:
        return f"target {target} was not restored by any step"
    if duration > case["horizon_s"] + 1e-6:
        return f"duration {duration} exceeds horizon {case['horizon_s']}"
    if abs(duration - case["total_duration_s"]) > 1e-6:
        return f"duration mismatch: steps {duration}, reported {case['total_duration_s']}"
    return None


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    verified = 0
    for index, case in enumerate(cases):
        problem = verify_plan(case)
        if problem:
            print(f"case {index} {case['target']}: FAIL: {problem}", file=sys.stderr)
        else:
            verified += 1
    print(f"plans={len(cases)} verified={verified}")
    if verified != len(cases):
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: verify.py <plans.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

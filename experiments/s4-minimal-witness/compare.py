#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare exact fixture blockers: missing resource amount and verification pair."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-recursive-closure"))
from solve import RANK, Model, si  # noqa: E402

MISSING = re.compile(r"missing resource ([^:]+): need ([0-9.]+), have ([0-9.]+)")
UNVERIFIED = re.compile(r"unverified process ([^:]+): (\w+) < (\w+)")


def check(case: dict) -> str | None:
    model = Model(case["model"])
    reason = case["reasons"][0]
    restorers = [p for p in model.processes if case["target"] in p.get("restores", [])]
    if not restorers:
        return "no restorer"
    match = MISSING.match(reason)
    if match:
        resource, need, have = match.group(1), float(match.group(2)), float(match.group(3))
        actual_need = None
        for process in model.processes:
            for item in process.get("inputs", []):
                if item["resource"] == resource and actual_need is None:
                    actual_need = si(item["quantity"])
        actual_have = model.stock.get(resource, 0.0)
        if actual_need != need or abs(actual_have - have) > 1e-9:
            return f"missing amount mismatch: rust need={need} have={have}, python need={actual_need} have={actual_have}"
        return None
    match = UNVERIFIED.match(reason)
    if match:
        process_name, rust_actual, rust_required = match.group(1), match.group(2), match.group(3)
        process = next((p for p in model.processes if p["id"] == process_name), None)
        if process is None:
            return f"unknown process {process_name}"
        actual = process.get("verification", "declared").capitalize()
        if rust_actual != actual or RANK.get(actual.lower(), 0) >= RANK[rust_required.lower()]:
            return f"verification mismatch: rust {rust_actual} < {rust_required}, python {actual}"
        return None
    return f"unhandled reason {reason}"


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    matched = 0
    for case in cases:
        problem = check(case)
        if problem:
            print(f"MISMATCH {case['model']}:{case['target']}: {problem}")
        else:
            matched += 1
    print(f"witnesses={len(cases)} matched={matched} mismatched={len(cases) - matched}")
    return 0 if matched == len(cases) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: compare.py <fixtures.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Systematic negative mutations over every exported RESTORABLE plan."""

from __future__ import annotations

import copy
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-independent-plan"))
from verify import verify_plan  # noqa: E402


def mutations(case: dict):
    yield "drop_all_steps", {**case, "steps": []}
    unknown = copy.deepcopy(case)
    step = copy.deepcopy(unknown["steps"][-1])
    step["process"] = "does-not-exist"
    unknown["steps"] = [*unknown["steps"], step]
    yield "unknown_process", unknown
    yield "wrong_total_duration", {**case, "total_duration_s": case["total_duration_s"] + 1.0}
    yield "missing_target_restore", {**case, "target": "__missing_target__"}
    yield "short_horizon", {**case, "horizon_s": 1.0}


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    plans = [case for case in cases if case.get("outcome") == "RESTORABLE"]
    total = 0
    rejected = 0
    for case in plans:
        for name, mutant in mutations(case):
            total += 1
            problem = verify_plan(mutant)
            if problem:
                rejected += 1
            else:
                print(f"ACCEPTED {name} {case['model']}:{case['target']}")
    print(f"mutations={total} rejected={rejected}")
    return 0 if rejected == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: fuzz.py <plans.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

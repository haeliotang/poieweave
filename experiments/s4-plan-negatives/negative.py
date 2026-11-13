#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Mutation check: the independent plan verifier must reject bad plans."""

from __future__ import annotations

import copy
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-independent-plan"))
from verify import verify_plan  # noqa: E402


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    restorable = [case for case in cases if case["outcome"] == "RESTORABLE"]
    if len(restorable) < 3:
        print("need at least 3 restorable plans", file=sys.stderr)
        return 2
    mutations = []

    dropped = copy.deepcopy(restorable[0])
    dropped["steps"] = dropped["steps"][:-1]
    mutations.append(("dropped_step", dropped))

    wrong_duration = copy.deepcopy(restorable[1])
    wrong_duration["total_duration_s"] += 1.0
    mutations.append(("wrong_duration", wrong_duration))

    unknown_process = copy.deepcopy(restorable[2])
    step = copy.deepcopy(unknown_process["steps"][-1])
    step["process"] = "does-not-exist"
    unknown_process["steps"] = [*unknown_process["steps"], step]
    mutations.append(("unknown_process", unknown_process))

    rejected = 0
    for name, case in mutations:
        problem = verify_plan(case)
        if problem:
            rejected += 1
            print(f"{name}: rejected ({problem})")
        else:
            print(f"{name}: ACCEPTED (bad)")
    print(f"mutations={len(mutations)} rejected={rejected}")
    return 0 if rejected == len(mutations) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: negative.py <plans.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

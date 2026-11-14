#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the S2 time-semantics certificate invariants."""

from __future__ import annotations

import json
import sys


def main(path: str) -> int:
    certificate = json.loads(open(path, encoding="utf-8").read())
    problems: list[str] = []
    if certificate.get("schema") != "poie.time-certificate/1":
        problems.append("schema")
    dense = certificate.get("dense", {})
    instant = dense.get("instant_times_h", [])
    advanced = dense.get("event_driven_times_h", [])
    repair_h = dense.get("repair_s", 0.0) / 3600.0
    if len(instant) != 2 or len(advanced) != 2:
        problems.append("dense length")
    else:
        if advanced[0] != instant[0] + repair_h:
            problems.append("dense first")
        if advanced[1] != max(instant[1], advanced[0]) + repair_h:
            problems.append("dense second")
    for row in certificate.get("sparse", []):
        if row.get("instant_days") != row.get("event_driven_days"):
            problems.append("sparse")
    deadline = certificate.get("deadline", {})
    if deadline.get("instant_complete") != deadline.get("failure_day"):
        problems.append("deadline instant")
    if deadline.get("advanced_complete") != deadline.get("failure_day", 0) + deadline.get("repair_days", 0):
        problems.append("deadline advanced")
    if (deadline.get("instant_complete", 99) <= deadline.get("deadline_day", 0)) == (
        deadline.get("advanced_complete", 99) <= deadline.get("deadline_day", 0)
    ):
        problems.append("deadline flip")
    for name, keys in (("grease_boundary", 5), ("insert_boundary", 5)):
        boundary = certificate.get(name, {})
        stocks = boundary.get("stocks", [])
        old = boundary.get("instant_days", [])
        new = boundary.get("advanced_days", [])
        if not (len(stocks) == len(old) == len(new) == keys):
            problems.append(f"{name} length")
        elif new[-1] <= old[-1]:
            problems.append(f"{name} saturation")
    print("certificate=ok" if not problems else "certificate=failed")
    for problem in problems:
        print(problem)
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

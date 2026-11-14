#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently check the S3 concurrency certificate's makespan arithmetic."""

from __future__ import annotations

import json
import sys


def main(path: str) -> int:
    certificate = json.loads(open(path, encoding="utf-8").read())
    problems: list[str] = []
    if certificate.get("schema") != "poie.concurrency-certificate/1":
        problems.append("schema")
    rows = certificate.get("rows", [])
    if len(rows) != 2:
        problems.append("row count")
    for row in rows:
        layout = row.get("layout")
        pump = row.get("pump_s")
        lathe = row.get("lathe_s")
        sequential = row.get("sequential_s")
        parallel = row.get("parallel_s")
        overlap = row.get("occupancy_overlap")
        deadline = row.get("deadline_s")
        if sequential != pump + lathe:
            problems.append(f"{layout}: sequential")
        expected_parallel = sequential if overlap else max(pump, lathe)
        if parallel != expected_parallel:
            problems.append(f"{layout}: parallel")
        if row.get("sequential_meets") != (sequential <= deadline):
            problems.append(f"{layout}: sequential deadline")
        if row.get("parallel_meets") != (parallel <= deadline):
            problems.append(f"{layout}: parallel deadline")
    if [(row.get("layout"), row.get("occupancy_overlap")) for row in rows] != [
        ("one-robot", True),
        ("two-robots", False),
    ]:
        problems.append("layouts")
    print("certificate=ok" if not problems else "certificate=failed")
    for problem in problems:
        print(problem)
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

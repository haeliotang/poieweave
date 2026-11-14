#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent structural check of the tool-repair cycle and its unblocker."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MODEL = ROOT / "scenarios/base-08/model.toml"


def quantity(value: str) -> float:
    return float(value.split()[0])


def main(path: str) -> int:
    model = tomllib.loads(MODEL.read_text())
    processes = model["process"]
    repair = next(process for process in processes if "lathe-1" in process.get("restores", []))
    needs_nut = any(item["resource"] == "lathe-nut" for item in repair["inputs"])
    producer = next(process for process in processes if any(item["resource"] == "lathe-nut" for item in process.get("outputs", [])))
    uses_lathe = "lathe-1" in producer.get("occupies", [])
    expected = {
        "no-spare": ("blocked", "cycle:asset:lathe-1"),
        "one-spare": ("restorable", f"{quantity(repair['duration']) * 3600:.0f}s"),
    }
    problems: list[str] = []
    if not (needs_nut and uses_lathe):
        problems.append("structure")
    rows = 0
    for line in Path(path).read_text().splitlines()[1:]:
        case, stock, outcome, detail = line.split("\t")
        rows += 1
        want = expected.get(case)
        if want is None or (outcome, detail) != want:
            problems.append(f"{case}: {(outcome, detail)} != {want}")
    print(f"rows={rows} matched={rows - len(problems)}")
    for problem in problems:
        print(problem)
    return 0 if rows == 2 and not problems else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

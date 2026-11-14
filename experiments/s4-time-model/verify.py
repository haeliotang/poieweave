#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent dense-event time arithmetic, compared with Rust output."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVENT_TIMES_H = [0.0, 1.0]


def repair_hours() -> float:
    model = tomllib.loads((ROOT / "scenarios/base-08/model.toml").read_text())
    process = next(item for item in model["process"] if item["id"] == "replace-sleeve-p1")
    amount, unit = process["duration"].split()
    return float(amount) * {"h": 1.0, "min": 1.0 / 60.0, "s": 1.0 / 3600.0}[unit]


def expected_times() -> tuple[list[float], list[float]]:
    repair = repair_hours()
    clock = 0.0
    advanced = []
    for event in EVENT_TIMES_H:
        start = max(event, clock)
        advanced.append(round(start + repair, 6))
        clock = start + repair
    return EVENT_TIMES_H, advanced


def parse_list(text: str) -> list[float]:
    inner = text.strip("[]")
    return [float(item) for item in inner.split(",") if item.strip()]


def main(path: str) -> int:
    instant, advanced = expected_times()
    matched = 0
    rows = 0
    for line in Path(path).read_text().splitlines()[1:]:
        parts = line.split("\t")
        if len(parts) != 5 or parts[0] != "dense":
            continue
        rows += 1
        mode = parts[2]
        observed = parse_list(parts[3].replace(" ", ","))
        expected = instant if mode == "instant-repair" else advanced
        if [round(value, 6) for value in observed] == [round(value, 6) for value in expected]:
            matched += 1
        else:
            print(f"{mode}: observed={observed} expected={expected}")
    print(f"rows={rows} matched={matched}")
    return 0 if rows == 2 and matched == 2 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

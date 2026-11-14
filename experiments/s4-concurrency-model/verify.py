#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent scheduling model for two repairs with shared/separate robots."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEADLINE_S = 4.0 * 3600.0


def seconds(text: str) -> float:
    amount, unit = text.split()
    return float(amount) * {"s": 1.0, "min": 60.0, "h": 3_600.0}[unit]


def durations() -> tuple[float, float]:
    model = tomllib.loads((ROOT / "scenarios/base-08/model.toml").read_text())
    pump = next(process for process in model["process"] if process["id"] == "replace-sleeve-p1")
    lathe = next(process for process in model["process"] if process["id"] == "replace-lathe-nut")
    return seconds(pump["duration"]), seconds(lathe["duration"])


def main(path: str) -> int:
    pump_s, lathe_s = durations()
    sequential = pump_s + lathe_s
    rows = 0
    matched = 0
    for line in Path(path).read_text().splitlines()[1:]:
        layout, pump_text, lathe_text, deadline_text, seq_text, par_text = line.split("\t")
        rows += 1
        overlap = layout == "one-robot"
        parallel = sequential if overlap else max(pump_s, lathe_s)
        expected = (
            layout,
            f"{pump_s:.0f}",
            f"{lathe_s:.0f}",
            f"{DEADLINE_S:.0f}",
            str(sequential <= DEADLINE_S).lower(),
            str(parallel <= DEADLINE_S).lower(),
        )
        observed = (layout, pump_text, lathe_text, deadline_text, seq_text, par_text)
        if expected == observed:
            matched += 1
        else:
            print(f"{layout}: observed={observed} expected={expected}")
    print(f"rows={rows} matched={matched}")
    return 0 if rows > 0 and matched == rows else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

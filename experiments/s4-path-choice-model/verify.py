#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent check of cross-path cost choice against the TOML duration."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ALT_S = 4 * 3600


def repair_seconds() -> int:
    model = tomllib.loads((ROOT / "scenarios/base-08/model.toml").read_text())
    process = next(item for item in model["process"] if item["id"] == "replace-lathe-nut")
    amount, unit = process["duration"].split()
    return int(float(amount) * {"h": 3600, "min": 60, "s": 1}[unit])


def main(path: str) -> int:
    spare = repair_seconds()
    expected = {
        "cycle": ("blocked", "cycle:asset:lathe-1"),
        "spare-only": ("restorable", f"{spare}s"),
        "alt-only": ("restorable", f"{ALT_S}s"),
        "both": ("restorable", f"{spare}s"),
    }
    matched = 0
    rows = 0
    for line in Path(path).read_text().splitlines()[1:]:
        parts = line.split("\t")
        if len(parts) != 5:
            continue
        rows += 1
        case, _spare, _alt, outcome, detail = parts
        if expected.get(case) == (outcome, detail):
            matched += 1
        else:
            print(f"{case}: {(outcome, detail)} != {expected.get(case)}")
    print(f"rows={rows} matched={matched}")
    return 0 if rows == 4 and matched == 4 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

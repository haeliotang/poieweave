#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent threshold check for the Base-09 bronze counterfactual."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-recursive-closure"))
from solve import Model  # noqa: E402


def main(path: str) -> int:
    lines = [line.strip() for line in Path(path).read_text().splitlines() if line.strip()]
    if not lines or not lines[0].startswith("bronze_kg"):
        print("bad input", file=sys.stderr)
        return 2
    base = Model("scenarios/base-09/model.toml")
    matched = 0
    rows = lines[1:]
    for row in rows:
        quantity, rust = row.split("\t")
        model = Model("scenarios/base-09/model.toml")
        model.stock["bronze-bar"] = model.stock.get("bronze-bar", 0.0) + float(quantity)
        python = "RESTORABLE" if model.verdict("pump-p1", 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
        if python == rust:
            matched += 1
        else:
            print(f"MISMATCH bronze={quantity} rust={rust} python={python}")
    print(f"cases={len(rows)} matched={matched} mismatched={len(rows) - matched}")
    return 0 if matched == len(rows) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: compare.py <unblock.txt>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

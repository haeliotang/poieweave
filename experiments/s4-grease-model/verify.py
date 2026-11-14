#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent grease-stock repair model, compared with Rust output."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVENTS = [10.0, 20.0, 30.0]
HORIZON = 30.0


def quantity_kg(text: str) -> float:
    amount, unit = text.split()
    factors = {"g": 0.001, "kg": 1.0, "t": 1000.0}
    return float(amount) * factors[unit]


def parameters() -> tuple[float, float, float]:
    model = tomllib.loads((ROOT / "scenarios/base-08/model.toml").read_text())
    grease_stock = sum(
        quantity_kg(lot["quantity"]) for lot in model.get("lot", []) if lot["resource"] == "grease"
    )
    process = next(item for item in model["process"] if item["id"] == "replace-sleeve-p1")
    grease_cost = next(quantity_kg(item["quantity"]) for item in process["inputs"] if item["resource"] == "grease")
    return grease_stock, grease_cost, 0.0


def simulate(stock: float, cost: float) -> float:
    remaining = stock
    if remaining + 1e-9 < cost:
        return 0.0
    remaining -= cost
    for day in EVENTS:
        if remaining + 1e-9 < cost:
            return day
        remaining -= cost
    return HORIZON


def main(path: str) -> int:
    stock, base_cost, _ = parameters()
    rows = 0
    matched = 0
    for line in Path(path).read_text().splitlines()[1:]:
        stock_text, mode, days_text, _blocked = line.split("\t")
        rows += 1
        override = stock if stock_text == f"{stock:.2f}" else float(stock_text)
        cost = base_cost if mode == "cheap" else 0.20
        expected = simulate(override, cost)
        if round(expected) == round(float(days_text)):
            matched += 1
        else:
            print(f"stock={stock_text} mode={mode} rust={days_text} python={expected:.0f}")
    print(f"rows={rows} matched={matched}")
    return 0 if rows > 0 and matched == rows else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

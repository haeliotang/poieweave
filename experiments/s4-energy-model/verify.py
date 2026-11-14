#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent linear energy model, compared with Rust horizon output."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
J_PER_KWH = 3_600_000.0
HORIZON_DAYS = 90.0
CAPACITY_KWH = 200.0
CHARGE_KW = 10.0


def joules(text: str) -> float:
    amount, unit = text.split()
    value = float(amount)
    factors = {"J": 1.0, "kJ": 1_000.0, "MJ": 1_000_000.0, "kWh": J_PER_KWH, "Wh": 3_600.0}
    return value * factors[unit]


def seconds(text: str) -> float:
    amount, unit = text.split()
    value = float(amount)
    factors = {"s": 1.0, "min": 60.0, "h": 3_600.0}
    return value * factors[unit]


def model_horizon_days() -> float:
    model = tomllib.loads((ROOT / "scenarios/base-08/model.toml").read_text())
    initial = sum(
        joules(lot["quantity"]) for lot in model.get("lot", []) if lot["resource"] == "electric-energy"
    )
    recharge = next(process for process in model["process"] if process["id"] == "recharge-energy")
    output = next(item for item in recharge["outputs"] if item["resource"] == "electric-energy")
    raw_watts = joules(output["quantity"]) / seconds(recharge["duration"])
    inflow_watts = min(raw_watts, CHARGE_KW * 1000.0)
    return initial, inflow_watts


def main(path: str) -> int:
    initial, inflow_watts = model_horizon_days()
    matched = 0
    rows = 0
    for line in Path(path).read_text().splitlines()[1:]:
        load_text, days_text, blocked = line.split("\t")
        rows += 1
        load_kw = float(load_text)
        net_watts = inflow_watts - load_kw * 1000.0
        expected_days = HORIZON_DAYS if net_watts >= 0 else (initial / -net_watts) / 86_400.0
        if int(expected_days) == int(float(days_text)):
            matched += 1
        else:
            print(f"load={load_kw} rust={days_text} python={expected_days:.2f}")
        if net_watts < 0 and blocked != "energy_depleted":
            print(f"load={load_kw} blocker {blocked} != energy_depleted")
            matched -= 1
    print(f"rows={rows} matched={matched}")
    return 0 if rows > 0 and matched == rows else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

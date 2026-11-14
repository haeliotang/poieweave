#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent two-phase energy model with a PV failure at day 5."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
J_PER_KWH = 3_600_000.0
CAPACITY_KWH = 200.0
CHARGE_KW = 10.0
PV_FAILURE_DAY = 5.0
HORIZON_DAYS = 90.0
J_PER_DAY = 86_400.0


def joules(text: str) -> float:
    amount, unit = text.split()
    factors = {"J": 1.0, "kJ": 1_000.0, "MJ": 1_000_000.0, "kWh": J_PER_KWH, "Wh": 3_600.0}
    return float(amount) * factors[unit]


def seconds(text: str) -> float:
    amount, unit = text.split()
    return float(amount) * {"s": 1.0, "min": 60.0, "h": 3_600.0}[unit]


def parameters() -> tuple[float, float]:
    model = tomllib.loads((ROOT / "scenarios/base-08/model.toml").read_text())
    initial = sum(joules(lot["quantity"]) for lot in model.get("lot", []) if lot["resource"] == "electric-energy")
    recharge = next(process for process in model["process"] if process["id"] == "recharge-energy")
    output = next(item for item in recharge["outputs"] if item["resource"] == "electric-energy")
    inflow = min(joules(output["quantity"]) / seconds(recharge["duration"]), CHARGE_KW * 1000.0)
    return initial, inflow


def main(path: str) -> int:
    initial_j, inflow_w = parameters()
    initial_kwh = initial_j / J_PER_KWH
    inflow_kw = inflow_w / 1000.0
    rows = 0
    matched = 0
    for line in Path(path).read_text().splitlines()[1:]:
        load_text, days_text, blocked = line.split("\t")
        rows += 1
        load_kw = float(load_text)
        phase1_net = inflow_kw - load_kw
        energy = min(CAPACITY_KWH, max(0.0, initial_kwh - 0.5 + phase1_net * PV_FAILURE_DAY * 24.0))
        total_days = PV_FAILURE_DAY + energy / (load_kw * 24.0) if load_kw > 0 else HORIZON_DAYS
        if round(total_days) == round(float(days_text)):
            matched += 1
        else:
            print(f"load={load_kw} rust={days_text} python={total_days:.2f}")
        if blocked != "energy_depleted":
            print(f"load={load_kw} blocker {blocked} != energy_depleted")
            matched -= 1
    print(f"rows={rows} matched={matched}")
    return 0 if rows > 0 and matched == rows else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

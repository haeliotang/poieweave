#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent direct-restorer verdicts for small S1 cases.

This is deliberately smaller than the Rust closure: it checks a single direct
restorer, its inputs, occupiers, capabilities and duration. It does not search
production chains or recursive repairs.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QUANTITY = re.compile(r"^\s*([0-9]+(?:\.[0-9]+)?)\s*([A-Za-z/ ]+?)\s*$")
UNITS = {"kg": 1.0, "item": 1.0, "items": 1.0, "kwh": 3_600_000.0, "h": 3600.0, "min": 60.0, "s": 1.0}
RANK = {"declared": 0, "modeled": 1, "demonstrated": 2, "verified": 3}


def si(text: str) -> float:
    match = QUANTITY.match(text)
    if not match:
        raise ValueError(text)
    return float(match.group(1)) * UNITS[match.group(2).strip().lower()]


def load(relative: str) -> dict:
    with (ROOT / relative).open("rb") as handle:
        return tomllib.load(handle)


def direct_verdict(model: dict, target: str, horizon_s: float) -> str:
    assets = {asset["id"]: asset for asset in model.get("asset", [])}
    if target not in assets:
        return "BLOCKED:unknown"
    target_asset = dict(assets[target])
    target_asset["status"] = "failed"
    assets[target] = target_asset
    stock: dict[str, float] = {}
    for lot in model.get("lot", []):
        if lot.get("quality", "accepted") == "accepted":
            stock[lot["resource"]] = stock.get(lot["resource"], 0.0) + si(lot["quantity"])
    available = {name for name, asset in assets.items() if asset.get("status") == "available"}
    providers: dict[str, int] = {}
    for name in available:
        for capability, level in assets[name].get("capability_verification", {}).items():
            providers[capability] = max(providers.get(capability, 0), RANK.get(level, 0))
    restorers = [
        process
        for process in model.get("process", [])
        if target in process.get("restores", [])
    ]
    if not restorers:
        return "BLOCKED:no_restorer"
    for process in restorers:
        if RANK.get(process.get("verification", "declared"), 0) < RANK["modeled"]:
            continue
        duration = si(process["duration"]) if process.get("duration") else float("inf")
        if duration > horizon_s:
            continue
        if any(occupier not in available for occupier in process.get("occupies", [])):
            continue
        if any(providers.get(capability, 0) < RANK["modeled"] for capability in process.get("requires_capabilities", [])):
            continue
        if any(stock.get(item["resource"], 0.0) + 1e-9 < si(item["quantity"]) for item in process.get("inputs", [])):
            continue
        return "RESTORABLE"
    return "BLOCKED:no_feasible_restorer"


def main() -> int:
    cases = [
        ("scenarios/base-04/model.toml", "lathe-1", 7200.0),
        ("scenarios/base-01/model.toml", "lathe-1", 7200.0),
        ("scenarios/base-05/model.toml", "lathe-1", 7200.0),
        ("scenarios/base-05/model.toml", "filter-1", 43200.0),
        ("scenarios/base-05/model.toml", "pump-p1", 10800.0),
    ]
    print("case\tverdict")
    for relative, target, horizon in cases:
        verdict = direct_verdict(load(relative), target, horizon)
        print(f"{relative}:{target}\t{verdict}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

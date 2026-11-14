#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent recursive repair checker for selected S1 cases.

Searches direct restorers and production chains with explicit cycle guards. It is
still a study implementation: it does not model shared time across branches,
requires_any, or parallel occupancy.
"""

from __future__ import annotations

import copy
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


def add_outputs(process: dict, stock: dict) -> None:
    """Apply process outputs, including an optional process_yield split."""
    spec = process.get("process_yield") or {}
    for item in process.get("outputs", []):
        amount = si(item["quantity"])
        fraction = float(spec.get("fraction", 1.0)) if spec.get("output") == item["resource"] else 1.0
        stock[item["resource"]] = stock.get(item["resource"], 0.0) + amount * fraction
        if fraction < 1.0:
            reject = spec["reject"]
            stock[reject] = stock.get(reject, 0.0) + amount * (1.0 - fraction)


class Model:
    def __init__(self, relative: str) -> None:
        with (ROOT / relative).open("rb") as handle:
            raw = tomllib.load(handle)
        self.assets = {asset["id"]: asset for asset in raw.get("asset", [])}
        self.processes = raw.get("process", [])
        self.stock: dict[str, float] = {}
        for lot in raw.get("lot", []):
            if lot.get("quality", "accepted") == "accepted":
                self.stock[lot["resource"]] = self.stock.get(lot["resource"], 0.0) + si(lot["quantity"])
        self.capability_assets: dict[str, list[tuple[str, int]]] = {}
        for name, asset in self.assets.items():
            for capability, level in asset.get("capability_verification", {}).items():
                self.capability_assets.setdefault(capability, []).append((name, RANK.get(level, 0)))

    def available(self, state: dict, asset: str) -> bool:
        status = state["assets"].get(asset, self.assets.get(asset, {}).get("status"))
        return status == "available"

    def usable(self, asset: str, state: dict, visiting: tuple[str, ...] = ()) -> bool:
        if not self.available(state, asset):
            return False
        if asset in visiting:
            return True
        for dependency in self.assets.get(asset, {}).get("requires", []):
            if not self.usable(dependency, state, visiting + (asset,)):
                return False
        return True

    def process_ok(self, process: dict, state: dict, remaining: float) -> bool:
        if RANK.get(process.get("verification", "declared"), 0) < RANK["modeled"]:
            return False
        duration = si(process["duration"]) if process.get("duration") else float("inf")
        if duration > remaining:
            return False
        if any(not self.available(state, asset) for asset in process.get("occupies", [])):
            return False
        for capability in process.get("requires_capabilities", []):
            candidates = self.capability_assets.get(capability, [])
            if not any(
                rank >= RANK["modeled"] and self.usable(name, state)
                for name, rank in candidates
            ):
                return False
        return True

    def try_restore(self, asset: str, state: dict, remaining: float, stack: tuple[str, ...]) -> bool:
        if self.available(state, asset):
            return self.usable(asset, state)
        key = f"asset:{asset}"
        if key in stack:
            return False
        for dependency in self.assets.get(asset, {}).get("requires", []):
            if not self.try_restore(dependency, state, remaining, stack + (key,)):
                return False
        for process in self.processes:
            if asset not in process.get("restores", []):
                continue
            if not self.process_ok(process, state, remaining):
                continue
            trial = copy.deepcopy(state)
            okay = True
            for item in process.get("inputs", []):
                if not self.try_resource(item["resource"], si(item["quantity"]), trial, remaining, stack + (key,)):
                    okay = False
                    break
            if not okay:
                continue
            add_outputs(process, trial["stock"])
            trial["assets"][asset] = "available"
            state.clear()
            state.update(trial)
            return True
        return False

    def try_resource(self, resource: str, need: float, state: dict, remaining: float, stack: tuple[str, ...]) -> bool:
        key = f"resource:{resource}"
        if state["stock"].get(resource, 0.0) + 1e-9 >= need:
            state["stock"][resource] = state["stock"].get(resource, 0.0) - need
            return True
        if key in stack:
            return False
        for process in self.processes:
            outputs = [item for item in process.get("outputs", []) if item["resource"] == resource]
            if not outputs:
                continue
            if not self.process_ok(process, state, remaining):
                continue
            trial = copy.deepcopy(state)
            okay = True
            for item in process.get("inputs", []):
                if not self.try_resource(item["resource"], si(item["quantity"]), trial, remaining, stack + (key,)):
                    okay = False
                    break
            if not okay:
                continue
            add_outputs(process, trial["stock"])
            runs = 1
            while trial["stock"].get(resource, 0.0) + 1e-9 < need and runs < 20:
                before = trial["stock"].get(resource, 0.0)
                repeated = copy.deepcopy(trial)
                okay = True
                for item in process.get("inputs", []):
                    if not self.try_resource(item["resource"], si(item["quantity"]), repeated, remaining, stack + (key,)):
                        okay = False
                        break
                if not okay:
                    break
                add_outputs(process, repeated["stock"])
                if repeated["stock"].get(resource, 0.0) <= before + 1e-9:
                    break
                trial = repeated
                runs += 1
            if trial["stock"].get(resource, 0.0) + 1e-9 < need:
                continue
            trial["stock"][resource] -= need
            state.clear()
            state.update(trial)
            return True
        return False

    def verdict(self, target: str, horizon: float) -> str:
        state = {"stock": dict(self.stock), "assets": {name: asset.get("status") for name, asset in self.assets.items()}}
        state["assets"][target] = "failed"
        return "RESTORABLE" if self.try_restore(target, state, horizon, ()) else "BLOCKED"


def main() -> int:
    cases = [
        ("scenarios/base-01/model.toml", "pump-p1", 43200.0),
        ("scenarios/base-01/model.toml", "lathe-1", 7200.0),
        ("scenarios/base-04/model.toml", "lathe-1", 7200.0),
        ("scenarios/base-05/model.toml", "pump-p1", 10800.0),
        ("scenarios/base-05/model.toml", "filter-1", 43200.0),
    ]
    print("case\tverdict")
    for relative, target, horizon in cases:
        print(f"{relative}:{target}\t{Model(relative).verdict(target, horizon)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

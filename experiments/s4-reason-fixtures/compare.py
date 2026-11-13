#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare missing-resource and unverified-process reason classes."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-recursive-closure"))
from solve import RANK, Model, si  # noqa: E402


def rust_class(reason: str) -> str:
    if reason.startswith("no restorer"):
        return "no_restorer"
    if reason.startswith("missing resource"):
        return "missing_resource"
    if reason.startswith("unverified process"):
        return "unverified_process"
    if reason.startswith("cyclic"):
        return "cycle"
    return "other"


def python_class(model: Model, target: str, horizon: float) -> str:
    restorers = [p for p in model.processes if target in p.get("restores", [])]
    if not restorers:
        return "no_restorer"
    if all(RANK.get(p.get("verification", "declared"), 0) < RANK["modeled"] for p in restorers):
        return "unverified_process"
    for process in restorers:
        if RANK.get(process.get("verification", "declared"), 0) < RANK["modeled"]:
            continue
        if any(model.stock.get(item["resource"], 0.0) < si(item["quantity"]) for item in process.get("inputs", [])):
            return "missing_resource"
    return "cycle"


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    matched = 0
    for case in cases:
        model = Model(case["model"])
        rust = rust_class(case["reasons"][0])
        python = python_class(model, case["target"], case["horizon_s"])
        if rust == python:
            matched += 1
        else:
            print(f"MISMATCH {case['model']}:{case['target']} rust={rust} python={python}")
    print(f"cases={len(cases)} matched={matched} mismatched={len(cases) - matched}")
    return 0 if matched == len(cases) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: compare.py <fixtures.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

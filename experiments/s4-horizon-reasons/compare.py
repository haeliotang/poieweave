#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare reason classes for a deliberately short 900 s horizon."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-recursive-closure"))
from solve import Model, si  # noqa: E402

QUANTITY = re.compile(r"need ([0-9.]+)s")


def rust_class(reason: str) -> str:
    if reason.startswith("no restorer"):
        return "no_restorer"
    if reason.startswith("cyclic"):
        return "cycle"
    if reason.startswith("horizon exceeded"):
        return "horizon"
    if reason.startswith("missing resource"):
        return "missing_resource"
    return "other"


def python_class(model: Model, target: str, horizon: float) -> str:
    restorers = [p for p in model.processes if target in p.get("restores", [])]
    if not restorers:
        return "no_restorer"
    durations = [si(p["duration"]) for p in restorers if p.get("duration")]
    if durations and all(duration > horizon for duration in durations):
        return "horizon"
    return "cycle"


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    models: dict[str, Model] = {}
    matched = 0
    for case in cases:
        model = models.setdefault(case["model"], Model(case["model"]))
        rust = rust_class(case["reasons"][0])
        python = python_class(model, case["target"], case["horizon_s"])
        if rust == python:
            matched += 1
        else:
            print(f"MISMATCH {case['model']}:{case['target']} rust={rust} python={python} reason={case['reasons'][0]}")
    print(f"blocked={len(cases)} matched={matched} mismatched={len(cases) - matched}")
    return 0 if matched == len(cases) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: compare.py <horizon.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare blocked-case reason classes: no_restorer vs cycle."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-recursive-closure"))
from solve import Model  # noqa: E402


def rust_class(reason: str) -> str:
    if reason.startswith("no restorer"):
        return "no_restorer"
    if reason.startswith("cyclic"):
        return "cycle"
    return "other"


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    blocked = [case for case in cases if case.get("outcome") == "BLOCKED"]
    matched = 0
    for case in blocked:
        model = Model(case["model"])
        restorers = [p for p in model.processes if case["target"] in p.get("restores", [])]
        python = "no_restorer" if not restorers else "cycle"
        rust = rust_class(case["reasons"][0])
        if python == rust:
            matched += 1
        else:
            print(f"MISMATCH {case['model']}:{case['target']} rust={rust} python={python}")
    print(f"blocked={len(blocked)} matched={matched} mismatched={len(blocked) - matched}")
    return 0 if matched == len(blocked) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: compare.py <corpus.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

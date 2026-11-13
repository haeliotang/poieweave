#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare Rust S1 verdicts with the independent recursive Python checker."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-recursive-closure"))
from solve import Model  # noqa: E402


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    models: dict[str, Model] = {}
    matched = 0
    for case in cases:
        model = models.setdefault(case["model"], Model(case["model"]))
        python = model.verdict(case["target"], case["horizon_s"])
        if python == case["rust"]:
            matched += 1
        else:
            print(f"MISMATCH {case['model']}:{case['target']} rust={case['rust']} python={python}")
    print(f"cases={len(cases)} matched={matched} mismatched={len(cases) - matched}")
    return 0 if matched == len(cases) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: compare.py <corpus.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

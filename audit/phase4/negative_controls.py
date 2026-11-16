#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run N4 negative controls through the event comparator."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n4_compare", PHASE / "compare.py")
compare = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(compare)

EXPECTED = {
    "N4-01-failure-delayed": "EVENT_TIME_DISPLACEMENT",
    "N4-01-failure-ignored": "STATE_TRANSITION_MISMATCH",
    "N4-01-regen-tampered": "RESOURCE_LEDGER_DIVERGENCE",
    "N4-01-duration-tampered": "EVENT_TIME_DISPLACEMENT",
    "N4-03-order-tampered": "EVENT_ORDER_MISMATCH",
}


def main() -> int:
    results = []
    ok = True
    for name, expected in EXPECTED.items():
        case = "N4-03" if name.startswith("N4-03") else "N4-01"
        result = compare.compare_case(case, PHASE / "negative-controls" / f"{name}.csv")
        classes = {finding["class"] for finding in result["findings"]}
        detected = expected in classes
        ok = ok and detected
        results.append({"control": name, "expected": expected, "detected": detected,
                        "classes": sorted(classes), "first_divergence": result["first_divergence"]})
        print(name, "PASS" if detected else "FAIL", sorted(classes))
    output = PHASE / "negative-controls" / "results.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n4-negative-controls/1", "results": results}, indent=2) + "\n")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

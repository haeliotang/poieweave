#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Negative controls for N4-C repair interruption."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("n4_repair", PHASE / "compare_repair.py")
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

EXPECTED = {
    "N4-05-failure-delayed": "EVENT_TIME_DISPLACEMENT",
    "N4-05-failure-ignored": "STATE_TRANSITION_MISMATCH",
}
results = []
ok = True
for name, expected in EXPECTED.items():
    result = module.compare(PHASE / "repair" / f"{name}.csv")
    classes = {finding["class"] for finding in result["findings"]}
    detected = expected in classes
    ok = ok and detected
    results.append({"control": name, "expected": expected, "detected": detected, "classes": sorted(classes),
                    "first_divergence": result["first_divergence"]})
    print(name, "PASS" if detected else "FAIL", sorted(classes))
(PHASE / "repair" / "negative-results.json").write_text(
    json.dumps({"schema": "poie.s5v-n4c-negative-controls/1", "results": results}, indent=2) + "\n"
)
raise SystemExit(0 if ok else 1)

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Cross-check C harness and fmpy for the repair-interruption FMU."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
COLUMNS = ["time_s", "energy_J", "generated_J", "load_J", "repair_J", "curtailed_J", "depleted",
           "pvAvailable", "repairState", "robotAvailable", "robotFailureAppliedTime_s", "terminated"]


def rows(path: Path) -> dict[float, dict[str, float]]:
    with path.open(newline="") as handle:
        return {float(row["time_s"]): {key: float(row[key]) for key in COLUMNS} for row in csv.DictReader(handle)}


left = rows(PHASE / "repair/N4-05-harness-900.csv")
right = rows(PHASE / "repair/N4-05-fmpy-900.csv")
common = sorted(set(left) & set(right))
worst = max((abs(left[t][key] - right[t][key]) for t in common for key in COLUMNS if key != "time_s"), default=0.0)
ok = worst <= 1e-6 and max(left) == max(right) and len(left) == len(right)
(PHASE / "repair/importer-agreement.json").write_text(json.dumps(
    {"schema": "poie.s5v-n4c-importer-agreement/1", "rows": len(common), "terminal_s": max(left),
     "max_abs_difference": worst, "ok": ok}, indent=2) + "\n")
print("repair importer", ok, worst)
raise SystemExit(0 if ok else 1)

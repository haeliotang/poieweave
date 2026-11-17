#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""C harness vs fmpy agreement for N5 scheduler cases."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CONFIG = json.loads((PHASE / "config.json").read_text())
COLUMNS = ["time_s", "makespan_s", "completedCount", "interrupted", "failureAppliedTime_s", "t1Start_s", "t1End_s",
           "t1Robot", "t2Start_s", "t2End_s", "t2Robot", "pumpRestored", "latheRestored", "lastEventCode",
           "terminated"]


def rows(path: Path) -> dict[float, dict[str, float]]:
    with path.open(newline="") as handle:
        return {float(row["time_s"]): {key: float(row[key]) for key in COLUMNS} for row in csv.DictReader(handle)}


def main() -> int:
    results = []
    ok = True
    for case in CONFIG["cases"]:
        cid = case["id"]
        left = rows(PHASE / "external" / f"{cid}-harness-900.csv")
        right = rows(PHASE / "external" / f"{cid}-fmpy-900.csv")
        common = sorted(set(left) & set(right))
        worst = max((abs(left[t][key] - right[t][key]) for t in common for key in COLUMNS if key != "time_s"), default=0.0)
        case_ok = worst <= 1e-6 and len(left) == len(right) and max(left) == max(right)
        ok = ok and case_ok
        results.append({"case": cid, "rows": len(common), "terminal_s": max(left), "max_abs_difference": worst,
                        "ok": case_ok})
    output = PHASE / "scheduling" / "importer-agreement.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps({"schema": "poie.s5v-n5-importer-agreement/1", "results": results}, indent=2) + "\n")
    for result in results:
        print(result["case"], result["ok"], result["max_abs_difference"])
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare the C harness and fmpy outputs for the same N4 run."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CONFIG = json.loads((PHASE / "config.json").read_text())
COLUMNS = ["time_s", "energy_J", "generated_J", "load_J", "repair_J", "curtailed_J", "depleted",
           "pvAvailable", "failureAppliedTime_s", "terminated"]


def rows(path: Path) -> dict[float, dict[str, float]]:
    with path.open(newline="") as handle:
        return {float(row["time_s"]): {key: float(row[key]) for key in COLUMNS} for row in csv.DictReader(handle)}


def main() -> int:
    results = []
    ok = True
    for case in CONFIG["cases"]:
        cid = case["id"]
        for step in CONFIG["steps_s"]:
            harness_path = PHASE / "results" / f"{cid}-harness-{step}.csv"
            fmpy_path = PHASE / "results" / f"{cid}-fmpy-{step}.csv"
            if not harness_path.is_file() or not fmpy_path.is_file():
                continue
            left, right = rows(harness_path), rows(fmpy_path)
            common = sorted(set(left) & set(right))
            worst = {key: 0.0 for key in COLUMNS if key not in ("time_s", "terminated")}
            for time_s in common:
                for key in worst:
                    worst[key] = max(worst[key], abs(left[time_s][key] - right[time_s][key]))
            terminal_match = abs(max(left) - max(right)) <= 1e-6
            row_match = len(left) == len(right)
            case_ok = terminal_match and row_match and max(worst.values()) <= 1e-6
            ok = ok and case_ok
            results.append({"case": cid, "step_s": step, "rows": len(common), "terminal_match": terminal_match,
                            "row_count_match": row_match, "max_abs_difference": worst, "ok": case_ok})
    output = PHASE / "results" / "importer-agreement.json"
    output.write_text(json.dumps({"schema": "poie.s5v-n4-importer-agreement/1", "results": results}, indent=2) + "\n")
    for result in results:
        print(result["case"], result["step_s"], result["ok"], max(result["max_abs_difference"].values()))
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Extract external FMU Before/After event evidence from fmpy step-900 CSVs."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CONFIG = json.loads((PHASE / "config.json").read_text())


def rows(path: Path) -> list[dict[str, float]]:
    with path.open(newline="") as handle:
        return [{key: float(value) for key, value in row.items()} for row in csv.DictReader(handle)]


def main() -> int:
    out = PHASE / "external"
    out.mkdir(parents=True, exist_ok=True)
    for case in [entry for entry in CONFIG["cases"] if entry.get("failure_asset", "pv-1") == "pv-1"]:
        cid = case["id"]
        data = rows(PHASE / "results" / f"{cid}-fmpy-900.csv")
        events = []
        if case["failure_s"] is not None:
            at = float(case["failure_s"])
            before = max((row for row in data if row["time_s"] < at - 1e-9), key=lambda row: row["time_s"])
            after = min((row for row in data if row["time_s"] >= at - 1e-9), key=lambda row: row["time_s"])
            events.append({"event": "AssetFailure", "target": "pv-1", "scheduled_s": at,
                           "before": {"time_s": before["time_s"], "pvAvailable": before["pvAvailable"],
                                      "energy_J": before["energy_J"]},
                           "after": {"time_s": after["time_s"], "pvAvailable": after["pvAvailable"],
                                     "energy_J": after["energy_J"],
                                     "failureAppliedTime_s": after["failureAppliedTime_s"]}})
        repair = next(row for row in data if row["repair_J"] >= 1_800_000.0 - 1e-6)
        prior = max((row for row in data if row["time_s"] < repair["time_s"] - 1e-9), key=lambda row: row["time_s"], default=repair)
        events.append({"event": "RepairCompleted", "target": "pump-p1",
                       "before": {"time_s": prior["time_s"], "repair_J": prior["repair_J"],
                                  "energy_J": prior["energy_J"]},
                       "after": {"time_s": repair["time_s"], "repair_J": repair["repair_J"],
                                 "energy_J": repair["energy_J"]}})
        (out / f"{cid}-events.json").write_text(json.dumps({"case": cid, "events": events}, indent=2) + "\n")
        print(cid, "events", len(events))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

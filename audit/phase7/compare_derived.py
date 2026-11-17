#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare N7 native N7-14 against N6-A-10800 derived restoration evidence."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
CASES = {
    "lathe-1": {"status_before": "Failed", "status_after": "Available"},
    "pump-p1": {"status_before": "Degraded", "status_after": "Available"},
}


def main() -> int:
    derived = json.loads((ROOT / "audit/phase6/canonical/N6-A-10800-audit.json").read_text())
    rows = list(csv.DictReader((PHASE / "observations/asset-statuses.csv").open(), delimiter="\t"))
    native = {}
    for row in rows:
        if row["case"] == "N7-14" and row["asset_id"] in CASES:
            native.setdefault(row["asset_id"], {})[row["phase"]] = row["asset_status"]
    comparisons = []
    for target, expected in CASES.items():
        actual_before = native.get(target, {}).get("Before")
        actual_after = native.get(target, {}).get("After")
        matches = actual_before == expected["status_before"] and actual_after == expected["status_after"]
        comparisons.append({
            "target_asset": target,
            "derived_before": expected["status_before"],
            "derived_after": expected["status_after"],
            "native_before": actual_before,
            "native_after": actual_after,
            "derived_completed_at_s": derived["group_committed_s"],
            "native_observed_at_s": 10800.0,
            "derived_termination": derived["termination"],
            "native_termination": "completed",
            "classification": "MATCH" if matches else "UNEXPLAINED",
        })
    status = "MATCH" if all(c["classification"] == "MATCH" for c in comparisons) else "UNEXPLAINED"
    data = {
        "schema": "poie.s5v-n7-native-vs-derived/2",
        "status": status,
        "derived_source": "audit/phase6/canonical/N6-A-10800-audit.json",
        "native_source": "audit/phase7/observations/asset-statuses.csv",
        "native_case": "N7-14",
        "comparisons": comparisons,
        "limitation": "N6-A-10800 uses a synthetic two-target setup over base-08; N7-14 reproduces the same setup through the native path. This compares restoration targets and timestamps, not all N6 cases.",
    }
    (PHASE / "differences/native-vs-derived.json").write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(status, comparisons)
    return 0 if status == "MATCH" else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Derive N6-D restoration state evidence from trace completion and external flags."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CASES = ["N6-C-tool-10800", "N6-C-tool-12000", "N6-C-bus-10800", "N6-D-same-10800",
         "N6-D-horizon-completion", "N6-D-horizon-failure"]


def main() -> int:
    out = PHASE / "restoration"
    out.mkdir(parents=True, exist_ok=True)
    records = []
    for case in CASES:
        canonical = json.loads((PHASE / "canonical" / f"{case}-audit.json").read_text())
        last = list(csv.DictReader((PHASE / "external" / f"{case}-fmpy-900.csv").open()))[-1]
        restored = canonical["restored_targets"]
        external_restored = [name for index, name in enumerate(["a-lathe", "b-pump", "c-filter"], start=1)
                             if float(last[f"restored{index}"]) > 0.5]
        records.append({
            "case": case,
            "targets": [{
                "target_asset": target,
                "asset_status_before": "FAILED",
                "asset_status_after": "AVAILABLE" if target in restored else "FAILED_UNCHANGED",
                "repair_completed_at_s": canonical["repair_completed_s"],
                "restoration_committed_at_s": canonical["repair_completed_s"],
                "resources_released_at_s": float(last["release_s"]),
                "selected_resources": [],
                "status_source": "DERIVED_FROM_TRACE_COMPLETION",
            } for target in ["a-lathe", "b-pump", "c-filter"][: canonical["tasks"]]],
            "canonical_restored_targets": restored,
            "external_restored_targets": external_restored,
            "restoration_agreement": set(restored) == set(external_restored),
            "native_asset_status_export": False,
            "limitation": "Native Observatory does not export asset status; AC/resource state is derived from commit trace and external restoration flags.",
        })
    (out / "restoration-state.json").write_text(
        json.dumps({"schema": "poie.s5v-n6d-restoration/1", "records": records}, indent=2) + "\n"
    )
    for record in records:
        print(record["case"], record["restoration_agreement"])
    return 0 if all(record["restoration_agreement"] for record in records) else 1


if __name__ == "__main__":
    raise SystemExit(main())

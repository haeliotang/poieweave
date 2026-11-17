#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check N8 event identity, ordering and noninterference evidence."""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent


def main() -> int:
    problems = []
    rows = list(csv.DictReader((PHASE / "event-observations/events.csv").open(), delimiter="\t"))
    summary = list(csv.DictReader((PHASE / "event-observations/summary.csv").open(), delimiter="\t"))
    if not rows or not summary:
        problems.append("empty event evidence")
    seen_ids: set[tuple[str, str]] = set()
    per_scenario: dict[str, list[dict]] = {}
    for row in rows:
        event_key = (row["scenario"], row["event_sequence"])
        if event_key in seen_ids:
            pass
        seen_ids.add(event_key)
        if row["event_status"] != "Committed":
            problems.append("uncommitted event in accepted evidence")
        if not row["event_id"].startswith(f"ev-{int(row['event_sequence']):04d}-"):
            problems.append("event id/sequence mismatch")
        per_scenario.setdefault(row["scenario"], []).append(row)
    event_ids: dict[tuple[str, str], str] = {}
    for row in rows:
        key = (row["scenario"], row["event_sequence"])
        previous = event_ids.setdefault(key, row["event_id"])
        if previous != row["event_id"]:
            problems.append("conflicting event id for sequence")
    for scenario, entries in per_scenario.items():
        sequences = sorted({int(row["event_sequence"]) for row in entries})
        if sequences != list(range(1, len(sequences) + 1)):
            problems.append(f"{scenario}: non-contiguous sequence")
    same_time = [row for row in rows if row["scenario"] == "N8-03" and row["at_s"] == "10800.000"
                 and row["event_kind"] in ("RepairCompleted", "AssetFailure")]
    kinds = {(row["event_kind"], row["event_sequence"], row["event_batch_sequence"]) for row in same_time}
    if ("RepairCompleted", "2", "2") not in kinds or ("AssetFailure", "3", "3") not in kinds:
        problems.append("same-time event order")
    if any(row["direct_event_equal"] != "true" or row["direct_time_equal"] != "true" for row in summary):
        problems.append("noninterference")
    if problems:
        print("N8 event evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N8 event identities, ordering and noninterference: {len(summary)} scenarios, {len(rows)} event asset rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

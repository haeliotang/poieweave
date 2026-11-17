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


def validate(rows: list[dict], summary: list[dict]) -> list[str]:
    problems = []
    if not rows or not summary:
        problems.append("empty event evidence")
    per_scenario: dict[str, list[dict]] = {}
    for row in rows:
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
    key = {(row["scenario"], row["event_sequence"], row["asset_id"]): row for row in rows}
    completion = key.get(("N8-03", "2", "pump-p1"))
    failure = key.get(("N8-03", "3", "robot-m1"))
    if not completion or completion["status_before"] != "Degraded" or completion["status_after"] != "Available":
        problems.append("completion state transition")
    if not failure or failure["status_before"] != "Available" or failure["status_after"] != "Failed":
        problems.append("failure state transition")
    if not completion or completion["at_s"] != "10800.000" or not failure or failure["at_s"] != "10800.000":
        problems.append("same-time event timestamp")
    if any(row["event_batch_sequence"] != row["event_sequence"] for row in same_time):
        problems.append("same-time event batch identity")
    if any(row["direct_event_equal"] != "true" or row["direct_time_equal"] != "true" for row in summary):
        problems.append("noninterference")
    trajectories = json.loads((PHASE / "trajectory-comparisons/trajectories.json").read_text())["scenarios"]
    if len(trajectories) != 3 or any(not scenario["cross_model_agreement"] for scenario in trajectories):
        problems.append("cross-model trajectory agreement")
    for scenario in trajectories:
        if not scenario["canonical_contract_valid"] or not scenario["external_contract_valid"]:
            problems.append("trajectory contract validity")
    negatives = json.loads((PHASE / "negative-controls/trajectory-results.json").read_text())["results"]
    if not all(entry["detected"] for entry in negatives):
        problems.append("trajectory negative controls")
    return problems


def main() -> int:
    rows = list(csv.DictReader((PHASE / "event-observations/events.csv").open(), delimiter="\t"))
    summary = list(csv.DictReader((PHASE / "event-observations/summary.csv").open(), delimiter="\t"))
    problems = validate(rows, summary)
    if problems:
        print("N8 event evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N8 event identities, ordering and noninterference: {len(summary)} scenarios, {len(rows)} event asset rows")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

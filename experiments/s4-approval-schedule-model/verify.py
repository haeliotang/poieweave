#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent approval-schedule receipt/horizon model, compared with Rust."""

from __future__ import annotations

import sys
from pathlib import Path

EVENTS_DAYS = [10, 20]
HORIZON_DAYS = 30
SCHEDULE = [(0, "Approved"), (20, "Rejected")]


def decision_at(day: int) -> str:
    decision = "Approved"
    for at, value in SCHEDULE:
        if at <= day:
            decision = value
    return decision


def expected() -> dict[str, tuple[int, int, str]]:
    all_rows = {}
    receipts = 1
    survived = HORIZON_DAYS
    blocked = "none"
    for day in EVENTS_DAYS:
        if decision_at(day) == "Rejected":
            survived = day
            blocked = "approval_rejected:pump-p1"
            break
        receipts += 1
    all_rows["approved-all"] = (3, HORIZON_DAYS, "none")
    all_rows["reject-day20"] = (receipts, survived, blocked)
    all_rows["multi-approved-all"] = (3, HORIZON_DAYS, "none")
    all_rows["multi-reject-day20"] = (receipts, survived, f"potable-water:{blocked}")
    return all_rows


def main(path: str) -> int:
    expected_rows = expected()
    matched = 0
    rows = 0
    for line in Path(path).read_text().splitlines()[1:]:
        parts = line.split("\t")
        if len(parts) != 4:
            continue
        rows += 1
        name, receipts, days, blocked = parts
        want = expected_rows.get(name)
        if want is not None and (int(receipts), int(days), blocked) == want:
            matched += 1
        else:
            print(f"{name}: {receipts}/{days}/{blocked} != {want}")
    print(f"rows={rows} matched={matched}")
    return 0 if rows == 4 and matched == 4 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

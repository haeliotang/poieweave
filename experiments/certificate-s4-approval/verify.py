#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the S4 approval-gate certificate."""

from __future__ import annotations

import json
import sys


def main(path: str) -> int:
    certificate = json.loads(open(path, encoding="utf-8").read())
    problems: list[str] = []
    if certificate.get("schema") != "poie.approval-certificate/1":
        problems.append("schema")
    plan_duration = certificate.get("plan_duration_s")
    cases = certificate.get("cases", [])
    if len(cases) != 7:
        problems.append("case count")
    for case in cases:
        name = case.get("name")
        decision = case.get("decision")
        lease_s = case.get("lease_s", 0)
        digest_ok = case.get("digest_ok")
        revoke_at_s = case.get("revoke_at_s")
        deny_at_s = case.get("deny_at_s")
        executed = case.get("executed")
        blocked = case.get("blocked_by", "")
        if decision == "Required":
            expected_executed, prefix = False, "approval_required"
        elif decision == "Rejected":
            expected_executed, prefix = False, "approval_rejected"
        elif lease_s < plan_duration:
            expected_executed, prefix = False, "approval_lease_expired"
        elif not digest_ok:
            expected_executed, prefix = False, "approval_digest_mismatch"
        elif revoke_at_s is not None:
            expected_executed, prefix = True, "approval_revoked"
        elif deny_at_s is not None:
            expected_executed, prefix = True, "approval_denied_at_event"
        else:
            expected_executed, prefix = True, "none"
        if executed != expected_executed:
            problems.append(f"{name}: executed")
        if prefix == "none" and blocked != "none":
            problems.append(f"{name}: blocker")
        if prefix != "none" and not blocked.startswith(prefix):
            problems.append(f"{name}: reason")
    schedule = certificate.get("schedule_cases", [])
    if len(schedule) != 2:
        problems.append("schedule count")
    else:
        expected = [
            ("approved-all", 3, 30.0, "none"),
            ("reject-day20", 2, 20.0, "approval_rejected:pump-p1"),
        ]
        for row, (name, receipts, days, blocked) in zip(schedule, expected):
            if (
                row.get("name") != name
                or row.get("receipts") != receipts
                or row.get("survived_days") != days
                or row.get("blocked_by") != blocked
            ):
                problems.append(f"schedule {name}")
    print("certificate=ok" if not problems else "certificate=failed")
    for problem in problems:
        print(problem)
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

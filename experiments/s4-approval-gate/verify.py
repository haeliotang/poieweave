#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent check of the S3 approval gate against exported cases."""

from __future__ import annotations

import json
import sys


def main() -> int:
    data = json.load(sys.stdin)
    plan_duration = data["plan_duration_s"]
    cases = data["cases"]
    checked = 0
    problems = []
    for case in cases:
        checked += 1
        name = case["name"]
        decision = case["decision"]
        lease_s = case["lease_s"]
        digest_ok = case["digest_ok"]
        revoke_at_s = case["revoke_at_s"]
        executed = case["executed"]
        blocked = case["blocked_by"]
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
        else:
            expected_executed, prefix = True, "none"
        if executed != expected_executed:
            problems.append(f"{name}: executed {executed} != {expected_executed}")
        if prefix == "none":
            if blocked != "none":
                problems.append(f"{name}: blocked {blocked} != none")
        elif not blocked.startswith(prefix):
            problems.append(f"{name}: blocked {blocked} lacks {prefix}")
    for problem in problems:
        print(problem)
    print(f"cases={checked} ok={checked - len(problems)}")
    print("OK" if not problems else "FAILED")
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently replay every exported RESTORABLE plan in the corpus."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "s4-independent-plan"))
from verify import verify_plan  # noqa: E402


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        cases = json.load(handle)
    plans = [case for case in cases if case.get("outcome") == "RESTORABLE"]
    verified = 0
    for case in plans:
        problem = verify_plan(case)
        if problem:
            print(f"MISMATCH {case['model']}:{case['target']}: {problem}")
        else:
            verified += 1
    print(f"plans={len(plans)} verified={verified}")
    return 0 if verified == len(plans) else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: verify_plans.py <corpus.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

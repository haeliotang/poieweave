#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Replay the native canonical observatory and verify event pre/post evidence."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = Path(__file__).resolve().parent / "native-trajectory.csv"


def main() -> int:
    result = subprocess.run(
        ["cargo", "run", "--locked", "-q", "-p", "poie-viability", "--bin", "poie-s5v-native-observatory"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        print(result.stderr[-500:], file=sys.stderr)
        return 1
    if result.stdout != EXPECTED.read_text():
        print("native trajectory differs from frozen evidence", file=sys.stderr)
        return 1
    rows = [line.split("\t") for line in result.stdout.strip().splitlines()[3:]]
    before = next(row for row in rows if row[0] == "10800" and row[10] == "Before")
    after = next(row for row in rows if row[0] == "10800" and row[10] == "After")
    if before[2] != "38.500000" or after[2] != "38.000000":
        print("event pre/post energy mismatch", file=sys.stderr)
        return 1
    if before[5] != "0.000000" or after[5] != "0.500000":
        print("event pre/post repair mismatch", file=sys.stderr)
        return 1
    print("ok native trajectory replay and event pre/post")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

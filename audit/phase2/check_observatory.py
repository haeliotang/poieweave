#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Replay the canonical trajectory observatory and compare with frozen evidence."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = ROOT / "audit/phase2/canonical-trajectory.csv"


def main() -> int:
    completed = subprocess.run(
        ["cargo", "run", "--locked", "-q", "-p", "poie-viability", "--bin", "poie-s5v-canonical-observatory"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        print(f"observatory replay failed: {completed.stderr[-500:]}", file=sys.stderr)
        return 1
    expected = EXPECTED.read_text()
    if completed.stdout != expected:
        print("canonical trajectory differs from frozen evidence", file=sys.stderr)
        return 1
    print("ok canonical trajectory replay")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

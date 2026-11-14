#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Ensure every experiment cited by the claims ledger exists with evidence files."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs/research/claims-ledger.zh-CN.md"


def main() -> int:
    text = LEDGER.read_text()
    cited = sorted(set(re.findall(r"experiments/([A-Za-z0-9_.\-]+)", text)))
    problems: list[str] = []
    for name in cited:
        directory = ROOT / "experiments" / name
        if not directory.is_dir():
            problems.append(f"missing dir {name}")
            continue
        for required in ("PREREG.md", "REPORT.md", "expected.txt"):
            if not (directory / required).is_file():
                problems.append(f"{name} missing {required}")
    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1
    print(f"ok claims: {len(cited)} experiments cited with evidence files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

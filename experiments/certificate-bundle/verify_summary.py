#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify that the one-page summary covers every bundle layer and hash."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main(bundle_path: str, summary_path: str) -> int:
    bundle = json.loads(Path(bundle_path).read_text())
    summary = Path(summary_path).read_text()
    problems: list[str] = []
    if bundle.get("claim_scope", "") not in summary:
        problems.append("claim scope")
    for row in bundle.get("certificates", []):
        layer = row["layer"]
        if f"| {layer} |" not in summary:
            problems.append(f"{layer} row")
        if row["sha256"][:16] not in summary:
            problems.append(f"{layer} hash")
    if "not a" not in summary:
        problems.append("disclaimer")
    if problems:
        print(f"summary=FAIL problems={problems}")
        return 1
    print("summary=ok")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("usage: verify_summary.py <bundle.json> <summary.md>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2]))

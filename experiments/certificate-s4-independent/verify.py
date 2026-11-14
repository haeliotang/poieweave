#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the S4 independent-model certificate against expected files."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def counts(text: str) -> tuple[int, int]:
    values: dict[str, int] = {}
    for token in text.split():
        if "=" in token:
            key, value = token.split("=", 1)
            try:
                values[key] = int(value)
            except ValueError:
                pass
    return values.get("rows", 0), values.get("matched", 0)


def main(path: str) -> int:
    certificate = json.loads(open(path, encoding="utf-8").read())
    problems: list[str] = []
    if certificate.get("schema") != "poie.independent-certificate/1":
        problems.append("schema")
    checks = certificate.get("checks", [])
    if len(checks) != 7:
        problems.append("check count")
    for check in checks:
        relative = check.get("expected", "")
        if not relative.startswith("experiments/"):
            problems.append(f"{check.get('name')}: path")
            continue
        target = ROOT / relative
        if not target.is_file():
            problems.append(f"{check.get('name')}: missing")
            continue
        rows, matched = counts(target.read_text())
        if rows != check.get("cases") or matched != check.get("matched"):
            problems.append(f"{check.get('name')}: counts")
        if matched != rows:
            problems.append(f"{check.get('name')}: mismatch")
    print("certificate=ok" if not problems else "certificate=failed")
    for problem in problems:
        print(problem)
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the regeneration certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = [
    ("scenarios/base-05/model.toml", 40.0),
    ("scenarios/base-08/model.toml", 60.0),
]


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.regeneration-certificate/1":
        problems.append("schema")
    rows = certificate.get("rows", [])
    if len(rows) != 2:
        problems.append("row count")
    for row, (relative, days) in zip(rows, EXPECTED):
        if row.get("model") != relative or row.get("survived_days") != days:
            problems.append(f"row {relative}")
            continue
        target = ROOT / relative
        if not target.is_file():
            problems.append(f"{relative} missing")
            continue
        digest = hashlib.sha256(target.read_bytes()).hexdigest()
        if digest != row.get("model_sha256"):
            problems.append(f"{relative} hash")
    if "extends" not in certificate.get("conclusion", ""):
        problems.append("conclusion")
    if not certificate.get("claim_scope", "").startswith("model-level regeneration result only"):
        problems.append("claim scope")
    if problems:
        print(f"certificate=FAIL problems={problems}")
        return 1
    print("certificate=ok")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: verify.py <certificate.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

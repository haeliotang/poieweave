#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the typed S2 metrics certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.metrics-certificate/1":
        problems.append("schema")
    for key, relative, days in (
        ("base05", "scenarios/base-05/model.toml", 40.0),
        ("base08", "scenarios/base-08/model.toml", 60.0),
    ):
        row = certificate.get(key, {})
        if row.get("model") != relative or row.get("survival_horizon_days") != days:
            problems.append(key)
            continue
        target = ROOT / relative
        if not target.is_file() or hashlib.sha256(target.read_bytes()).hexdigest() != row.get("model_sha256"):
            problems.append(f"{key} hash")
    coverage = certificate.get("recovery_coverage", {})
    if coverage.get("recoverable") != 5 or coverage.get("total") != 10 or coverage.get("ratio") != 0.5:
        problems.append("coverage")
    if certificate.get("regenerative_capacity_days") != 20.0:
        problems.append("regenerative capacity")
    if not certificate.get("claim_scope", "").startswith("model-level metric definitions only"):
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

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the Base-05 site-load certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.viability-certificate/1":
        problems.append("schema")
    model_path = ROOT / certificate.get("model", "")
    if not model_path.is_file():
        problems.append("model path")
    else:
        digest = hashlib.sha256(model_path.read_bytes()).hexdigest()
        if digest != certificate.get("model_sha256"):
            problems.append("model hash")
    if certificate.get("horizon_days") != 90:
        problems.append("horizon")
    battery = certificate.get("battery", {})
    if battery.get("capacity_kwh") != 200.0 or battery.get("charge_kw") != 10.0:
        problems.append("battery")
    rows = {row["load_kw"]: row for row in certificate.get("rows", [])}
    if sorted(rows) != [0.0, 1.0, 2.0, 3.0, 4.0]:
        problems.append("load set")
    for load in (0.0, 1.0, 2.0):
        row = rows.get(load, {})
        if row.get("survived_days") != 40.0 or not str(row.get("blocked_by", "")).startswith("missing resource"):
            problems.append(f"material row {load}")
    for load, limit in ((3.0, 10.0), (4.0, 5.0)):
        row = rows.get(load, {})
        if not row.get("survived_days", 999) < limit or row.get("blocked_by") != "energy_depleted":
            problems.append(f"energy row {load}")
    if not certificate.get("claim_scope", "").startswith("model/simulation only"):
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

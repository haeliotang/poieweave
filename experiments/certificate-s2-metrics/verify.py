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
    sharpening = certificate.get("insert_sharpening", {})
    if sharpening.get("baseline_days") != 40 or sharpening.get("with_sharpening_days") != 60 or sharpening.get("gain_days") != 20:
        problems.append("insert sharpening")
    redundancy = certificate.get("redundancy", {})
    expected_redundancy = {
        "baseline_days": 60,
        "redundant_pump_days": 90,
        "shared_filter_days": 30,
        "filter_restorer_days": 90,
        "shared_robot_days": 35,
        "robot_restorer_days": 90,
    }
    if any(redundancy.get(key) != value for key, value in expected_redundancy.items()):
        problems.append("redundancy")
    yield_sensitivity = certificate.get("yield_sensitivity", {})
    if yield_sensitivity.get("yield_1_0_days") != 60 or yield_sensitivity.get("yield_0_8_days") != 60 or yield_sensitivity.get("yield_0_5_days") != 50:
        problems.append("yield sensitivity")
    boundary = certificate.get("recharge_boundary", {})
    if boundary.get("average_recharge_kw") != 2.5 or boundary.get("threshold_load_kw") != 2.6 or boundary.get("margin_kw") != 0.1:
        problems.append("recharge boundary")
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

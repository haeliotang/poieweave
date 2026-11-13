#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the Base-06 joint coverage certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.coverage-certificate/1":
        problems.append("schema")
    model_path = ROOT / certificate.get("model", "")
    if not model_path.is_file():
        problems.append("model path")
    else:
        digest = hashlib.sha256(model_path.read_bytes()).hexdigest()
        if digest != certificate.get("model_sha256"):
            problems.append("model hash")
    expected = {
        "total": 56,
        "independent_recoverable": 25,
        "joint_recoverable": 24,
        "optimistic_gap": 1,
        "max_failures": 2,
    }
    for key, value in expected.items():
        if certificate.get(key) != value:
            problems.append(key)
    if certificate.get("service") != "potable-water" or certificate.get("horizon_s") != 43200.0:
        problems.append("service/horizon")
    if certificate.get("gap_pairs") != [["pump-p1", "pump-p2"]]:
        problems.append("gap pair")
    if not certificate.get("claim_scope", "").startswith("model-level coverage only"):
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

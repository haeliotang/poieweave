#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the Base-06 repair-order certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.order-certificate/1":
        problems.append("schema")
    model = ROOT / certificate.get("model", "")
    if not model.is_file():
        problems.append("model path")
    elif hashlib.sha256(model.read_bytes()).hexdigest() != certificate.get("model_sha256"):
        problems.append("model hash")
    expected = [
        {"max_failures": 2, "recoverable_cases": 25, "standard_joint": 24, "best_joint": 24, "order_sensitive": 0},
        {"max_failures": 3, "recoverable_cases": 44, "standard_joint": 40, "best_joint": 40, "order_sensitive": 0},
    ]
    if certificate.get("rows") != expected:
        problems.append("rows")
    if "does not close" not in certificate.get("conclusion", ""):
        problems.append("conclusion")
    if not certificate.get("claim_scope", "").startswith("model-level ordering result only"):
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

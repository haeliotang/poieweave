#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the multi-service recovery certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments/s4-independent-receipt"))
from verify import verify  # noqa: E402


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.multi-recovery-certificate/1":
        problems.append("schema")
    model_path = ROOT / certificate.get("model", "")
    if not model_path.is_file():
        problems.append("model path")
    elif hashlib.sha256(model_path.read_bytes()).hexdigest() != certificate.get("model_sha256"):
        problems.append("model hash")
    if certificate.get("services") != ["potable-water", "workshop-operability"]:
        problems.append("services")
    if certificate.get("events") != [
        {"at_s": 864000.0, "asset": "pump-p1"},
        {"at_s": 1728000.0, "asset": "lathe-1"},
    ]:
        problems.append("events")
    receipts = certificate.get("receipts", [])
    if len(receipts) != 3 or not all(verify(receipt) for receipt in receipts):
        problems.append("receipts")
    elif any(a.get("inventory_after") != b.get("inventory_before") for a, b in zip(receipts, receipts[1:])):
        problems.append("inventory chain")
    if not certificate.get("claim_scope", "").startswith("isolated simulation only"):
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

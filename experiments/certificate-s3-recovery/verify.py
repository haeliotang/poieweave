#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the S3 recovery certificate."""

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
    if certificate.get("schema") != "poie.recovery-certificate/1":
        problems.append("schema")
    model_path = ROOT / certificate.get("model", "")
    if not model_path.is_file():
        problems.append("model path")
    else:
        digest = hashlib.sha256(model_path.read_bytes()).hexdigest()
        if digest != certificate.get("model_sha256"):
            problems.append("model hash")
    if certificate.get("service") != "potable-water" or certificate.get("horizon_days") != 30:
        problems.append("service/horizon")
    approval = certificate.get("approval", {})
    if approval.get("decision") != "approved" or approval.get("lease_s") != 28800.0:
        problems.append("approval")
    events = certificate.get("events", [])
    if events != [{"at_s": 864000.0, "asset": "pump-p1"}]:
        problems.append("events")
    receipts = certificate.get("receipts", [])
    if len(receipts) != 2 or not all(verify(receipt) for receipt in receipts):
        problems.append("receipts")
    elif receipts[0].get("inventory_after") != receipts[1].get("inventory_before"):
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

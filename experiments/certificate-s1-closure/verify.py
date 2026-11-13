#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the S1 closure certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments/s4-independent-plan"))
from verify import verify_plan  # noqa: E402


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.closure-certificate/1":
        problems.append("schema")
    model_path = ROOT / certificate.get("model", "")
    if not model_path.is_file():
        problems.append("model path")
    else:
        digest = hashlib.sha256(model_path.read_bytes()).hexdigest()
        if digest != certificate.get("model_sha256"):
            problems.append("model hash")
    if certificate.get("target") != "lathe-1" or certificate.get("horizon_s") != 7200.0:
        problems.append("target/horizon")
    if certificate.get("outcome") != "RESTORABLE":
        problems.append("outcome")
    plan_problem = verify_plan(
        {
            "model": certificate.get("model"),
            "outcome": "RESTORABLE",
            "target": certificate.get("target"),
            "horizon_s": certificate.get("horizon_s"),
            "steps": certificate.get("steps"),
            "total_duration_s": certificate.get("total_duration_s"),
        }
    )
    if plan_problem:
        problems.append(f"plan: {plan_problem}")
    if not certificate.get("claim_scope", "").startswith("model-level closure only"):
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

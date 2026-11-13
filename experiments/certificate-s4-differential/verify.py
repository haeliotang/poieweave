#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the S4 differential evidence certificate."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(path: str) -> int:
    certificate = json.loads(Path(path).read_text())
    problems: list[str] = []
    if certificate.get("schema") != "poie.s4-certificate/1":
        problems.append("schema")
    for key, expected in (
        ("differential_expected", "experiments/s4-differential-corpus/expected.txt"),
        ("plans_expected", "experiments/s4-differential-corpus/expected-plans.txt"),
    ):
        relative = certificate.get(key)
        target = ROOT / str(relative)
        if not target.is_file():
            problems.append(f"{key} missing")
            continue
        digest = hashlib.sha256(target.read_bytes()).hexdigest()
        if digest != certificate.get(f"{key}_sha256"):
            problems.append(f"{key} hash")
    if certificate.get("differential_summary") != "cases=112 matched=112 mismatched=0":
        problems.append("differential summary")
    if certificate.get("plans_summary") != "plans=26 verified=26":
        problems.append("plans summary")
    if not certificate.get("claim_scope", "").startswith("independent implementation evidence only"):
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

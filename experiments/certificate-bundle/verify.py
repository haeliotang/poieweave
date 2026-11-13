#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the S1/S2/S3 certificate bundle."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = {
    "S1": ("experiments/certificate-s1-closure/certificate.json", "poie.closure-certificate/1"),
    "S2": ("experiments/certificate-s2-load/certificate.json", "poie.viability-certificate/1"),
    "S2-joint": ("experiments/certificate-s2-joint/certificate.json", "poie.coverage-certificate/1"),
    "S2-regen": ("experiments/certificate-s2-regeneration/certificate.json", "poie.regeneration-certificate/1"),
    "S3": ("experiments/certificate-s3-recovery/certificate.json", "poie.recovery-certificate/1"),
    "S4": ("experiments/certificate-s4-differential/certificate.json", "poie.s4-certificate/1"),
}


def main(path: str, root_arg: str | None = None) -> int:
    root = Path(root_arg) if root_arg else ROOT
    bundle = json.loads(Path(path).read_text())
    problems: list[str] = []
    if bundle.get("schema") != "poie.certificate-bundle/1":
        problems.append("schema")
    rows = {row.get("layer"): row for row in bundle.get("certificates", [])}
    if sorted(rows) != ["S1", "S2", "S2-joint", "S2-regen", "S3", "S4"]:
        problems.append("layers")
    for layer, (relative, schema) in EXPECTED.items():
        row = rows.get(layer, {})
        if row.get("path") != relative or row.get("schema") != schema:
            problems.append(f"{layer} identity")
            continue
        path = root / relative
        if not path.is_file():
            problems.append(f"{layer} missing")
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest != row.get("sha256"):
            problems.append(f"{layer} hash")
    if not bundle.get("claim_scope", "").startswith("model/simulation certificates only"):
        problems.append("claim scope")
    if problems:
        print(f"bundle=FAIL problems={problems}")
        return 1
    print("bundle=ok")
    return 0


if __name__ == "__main__":
    if len(sys.argv) not in (2, 3):
        print("usage: verify.py <bundle.json> [root]", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2] if len(sys.argv) == 3 else None))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Negative test: a modified certificate must fail bundle verification."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VERIFY = ROOT / "experiments/certificate-bundle/verify.py"
BUNDLE = "experiments/certificate-bundle/bundle.json"
CERTIFICATES = [
    "experiments/certificate-s1-closure/certificate.json",
    "experiments/certificate-s2-load/certificate.json",
    "experiments/certificate-s2-joint/certificate.json",
    "experiments/certificate-s2-regeneration/certificate.json",
    "experiments/certificate-s2-metrics/certificate.json",
    "experiments/certificate-s2-order/certificate.json",
    "experiments/certificate-s3-recovery/certificate.json",
    "experiments/certificate-s3-multi/certificate.json",
    "experiments/certificate-s4-differential/certificate.json",
]


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / BUNDLE).parent.mkdir(parents=True)
        shutil.copy2(ROOT / BUNDLE, root / BUNDLE)
        for relative in CERTIFICATES:
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, target)
        clean = subprocess.run([sys.executable, str(VERIFY), str(root / BUNDLE), str(root)], capture_output=True, text=True)
        if clean.returncode != 0 or "bundle=ok" not in clean.stdout:
            print(f"clean bundle failed: {clean.stdout} {clean.stderr}", file=sys.stderr)
            return 1
        tampered = root / CERTIFICATES[0]
        tampered.write_bytes(tampered.read_bytes() + b" ")
        dirty = subprocess.run([sys.executable, str(VERIFY), str(root / BUNDLE), str(root)], capture_output=True, text=True)
        if dirty.returncode == 0:
            print("tampered bundle unexpectedly passed", file=sys.stderr)
            return 1
        print("tamper=detected")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())

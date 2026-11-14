#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the certificate bundle in a temporary copy without the workspace."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUNDLE = ROOT / "experiments/certificate-bundle"
REQUIRED_DIRS = [
    "certificate-s1-closure",
    "certificate-s2-load",
    "certificate-s2-joint",
    "certificate-s2-regeneration",
    "certificate-s2-metrics",
    "certificate-s2-order",
    "certificate-s2-time",
    "certificate-s3-recovery",
    "certificate-s3-multi",
    "certificate-s3-concurrency",
    "certificate-s4-differential",
    "certificate-s4-approval",
    "certificate-s4-independent",
]


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, capture_output=True, text=True)


def main() -> int:
    with tempfile.TemporaryDirectory() as raw:
        room = Path(raw)
        experiments = room / "experiments"
        experiments.mkdir()
        shutil.copytree(BUNDLE, experiments / "certificate-bundle")
        for name in REQUIRED_DIRS:
            shutil.copytree(ROOT / "experiments" / name, experiments / name)
        shutil.copytree(ROOT / "scenarios", room / "scenarios")
        bundle = experiments / "certificate-bundle" / "bundle.json"
        verified = run(sys.executable, str(experiments / "certificate-bundle" / "verify.py"), str(bundle), str(room))
        if verified.returncode != 0:
            print(verified.stdout, verified.stderr)
            return 1
        summary = run(
            sys.executable,
            str(experiments / "certificate-bundle" / "verify_summary.py"),
            str(bundle),
            str(experiments / "certificate-bundle" / "SUMMARY.md"),
        )
        if summary.returncode != 0:
            print(summary.stdout, summary.stderr)
            return 1
    print("clean-room=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

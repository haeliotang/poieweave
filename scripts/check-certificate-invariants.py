#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run every certificate verifier plus bundle, summary and clean-room checks."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, capture_output=True, text=True, check=False)


def main() -> int:
    failures: list[str] = []
    checked = 0
    for verifier in sorted((ROOT / "experiments").glob("certificate-*/verify.py")):
        certificate = verifier.parent / "certificate.json"
        if not certificate.is_file():
            continue
        result = run(sys.executable, str(verifier), str(certificate))
        checked += 1
        if result.returncode != 0:
            failures.append(str(verifier.parent.name))
            print(result.stdout, result.stderr)
    bundle = ROOT / "experiments/certificate-bundle/bundle.json"
    summary = ROOT / "experiments/certificate-bundle/SUMMARY.md"
    for name, args in (
        ("bundle", ("experiments/certificate-bundle/verify.py", str(bundle))),
        (
            "summary",
            ("experiments/certificate-bundle/verify_summary.py", str(bundle), str(summary)),
        ),
        ("tamper", ("experiments/certificate-bundle-tamper/tamper.py",)),
        ("clean-room", ("experiments/certificate-clean-room/clean_room.py",)),
    ):
        result = run(sys.executable, str(ROOT / args[0]), *args[1:])
        checked += 1
        if result.returncode != 0:
            failures.append(name)
            print(result.stdout, result.stderr)
    if failures:
        print(f"certificate invariants failed: {', '.join(failures)}", file=sys.stderr)
        return 1
    print(f"ok certificate invariants: {checked} checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

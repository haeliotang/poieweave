#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify the frozen v1 certificates at their baseline commit."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASELINE = json.loads((Path(__file__).resolve().parent / "v1-frozen.json").read_text())["baseline_commit"]


def run(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, check=False)


def main() -> int:
    if run(["git", "cat-file", "-e", f"{BASELINE}^{{commit}}"], ROOT).returncode != 0:
        print(f"baseline commit {BASELINE[:8]} is unavailable; fetch full git history", file=sys.stderr)
        return 1
    with tempfile.TemporaryDirectory() as directory:
        worktree = Path(directory) / "v1"
        added = run(["git", "worktree", "add", "--detach", str(worktree), BASELINE], ROOT)
        if added.returncode != 0:
            print(added.stderr, file=sys.stderr)
            return 1
        try:
            for command in (
                [sys.executable, "scripts/verify-canonical-claim-certificates.py"],
                [sys.executable, "scripts/check-canonical-claim-certificate-tamper.py"],
                [sys.executable, "scripts/build-canonical-s2-certificates.py", "--check"],
                [sys.executable, "scripts/build-canonical-metrics-certificate.py", "--check"],
                [sys.executable, "scripts/check-canonical-certificate-tamper.py"],
            ):
                result = run(command, worktree)
                if result.returncode != 0:
                    print(f"historical v1 verification failed: {' '.join(command)}", file=sys.stderr)
                    print(result.stdout, result.stderr, file=sys.stderr)
                    return 1
        finally:
            run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
    print(f"ok v1 historical verification at {BASELINE[:8]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify frozen v2 certificate bytes and run historical v2 verification."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).resolve().parent / "v2-frozen.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, check=False)


def main() -> int:
    data = json.loads(MANIFEST.read_text())
    problems = []
    for entry in data["certificate_files"] + [data["v2_index"]]:
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            problems.append(f"v2 evidence changed or missing: {entry['path']}")
    if problems:
        print("v2 freeze FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    baseline = data["baseline_commit"]
    if run(["git", "cat-file", "-e", f"{baseline}^{{commit}}"], ROOT).returncode != 0:
        print(f"baseline {baseline[:8]} unavailable; fetch full history", file=sys.stderr)
        return 1
    with tempfile.TemporaryDirectory() as directory:
        worktree = Path(directory) / "v2"
        added = run(["git", "worktree", "add", "--detach", str(worktree), baseline], ROOT)
        if added.returncode != 0:
            print(added.stderr, file=sys.stderr)
            return 1
        try:
            for command in (
                [sys.executable, "scripts/build-canonical-claim-certificates-v2.py", "--check"],
                [sys.executable, "scripts/verify-canonical-claim-certificates-v2.py"],
                [sys.executable, "scripts/check-canonical-claim-certificate-tamper-v2.py"],
            ):
                result = run(command, worktree)
                if result.returncode != 0:
                    print(f"historical v2 verification failed: {' '.join(command)}", file=sys.stderr)
                    print(result.stdout, result.stderr, file=sys.stderr)
                    return 1
        finally:
            run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
    print(f"ok v2 certificate freeze: {len(data['certificate_files'])} files; historical verification at {baseline[:8]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Fail when the status snapshot's artifact/CI-path counts are stale."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STATUS = ROOT / "docs/status/status-snapshot.zh-CN.md"


def main() -> int:
    replay = subprocess.run(
        [sys.executable, str(ROOT / "scripts/check-experiment-replays.py")],
        capture_output=True,
        text=True,
        check=False,
    )
    match = re.search(r"OK: (\d+) CI experiment paths exist; (\d+) generated artifacts", replay.stdout)
    if match is None:
        print(replay.stdout, replay.stderr)
        return 1
    paths, artifacts = (int(value) for value in match.groups())
    status_match = re.search(r"生成型实验产物 \| \*\*(\d+)\*\* 个：(\d+) 个 CI", STATUS.read_text())
    if status_match is None:
        print("status row not found", file=sys.stderr)
        return 1
    status_artifacts, status_paths = (int(value) for value in status_match.groups())
    if (status_artifacts, status_paths) != (artifacts, paths):
        print(
            f"status mismatch: status={status_artifacts}/{status_paths} actual={artifacts}/{paths}",
            file=sys.stderr,
        )
        return 1
    print(f"ok status counts: {artifacts} artifacts, {paths} CI paths")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check Phase 2 challenge artifacts are internally consistent."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    result = json.loads((HERE / "result.json").read_text())
    manifest = json.loads((HERE / "manifest.json").read_text())
    problems = []
    if result["gate2"] != "PASS" or result["problems"]:
        problems.append("gate2")
    if result["runs"]["tampered"]["detected"] is not True:
        problems.append("tamper detection")
    if result["runs"]["coarse"]["max_abs_error_J"] > 1.0:
        problems.append("coarse error")
    if result["runs"]["fine"]["max_abs_error_J"] > 1.0:
        problems.append("fine error")
    for entry in manifest["files"]:
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            problems.append(f"manifest {entry['path']}")
    for name in ("REPORT.zh-CN.md", "result.json", "config.json", "canonical-reference.txt"):
        if not (HERE / name).is_file():
            problems.append(f"missing {name}")
    if problems:
        print("phase2 challenge artifacts FAIL: " + ", ".join(problems))
        return 1
    print("ok phase2 challenge artifacts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify that frozen v1 certificate bytes and runtime baseline are unchanged."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).resolve().parent / "v1-frozen.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    data = json.loads(MANIFEST.read_text())
    problems = []
    for entry in data["certificate_files"]:
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            problems.append(f"certificate changed or missing: {entry['path']}")
    changed_runtime = []
    for entry in data["runtime_sources"]:
        path = ROOT / entry["path"]
        if not path.is_file() or sha(path) != entry["sha256"]:
            changed_runtime.append(entry["path"])
    if problems:
        print("v1 certificate freeze FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok v1 certificate freeze: {len(data['certificate_files'])} files")
    if changed_runtime:
        print("historical runtime baseline at {} differs as expected after v2 migration: {}".format(
            data["baseline_commit"][:8], ", ".join(changed_runtime)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

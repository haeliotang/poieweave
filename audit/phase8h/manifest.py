#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze tracked N8-H evidence from the Git index."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
PREFIX = "audit/phase8h/"
FILES = {"PREREG.md", "SEMANTICS.md", "REPORT.zh-CN.md", "EVIDENCE.md",
         "old-evidence.json", "false-pass.json", "fmpy_schedule.py", "compare_strict.py",
         "negative_controls.py", "check.py"}
DIRS = {"external/", "comparisons/", "negative-controls/"}
EXCLUDE = {"manifest.json", "execution-manifest.json"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def allowed(relative: str) -> bool:
    if relative in FILES:
        return True
    if Path(relative).name in EXCLUDE or relative.endswith((".log", ".meta.json")):
        return False
    return any(relative.startswith(directory) for directory in DIRS)


def main() -> int:
    tracked = subprocess.run(["git", "ls-files", "-z", PREFIX], cwd=ROOT, check=True, capture_output=True)
    files = []
    for entry in tracked.stdout.split(b"\0"):
        if not entry:
            continue
        path = entry.decode()
        relative = path[len(PREFIX):]
        if allowed(relative) and (ROOT / path).is_file():
            files.append({"path": path, "sha256": sha(ROOT / path), "bytes": (ROOT / path).stat().st_size})
    manifest = {"schema": "poie.s5v-n8h-evidence/1", "kind": "FROZEN_TRACKED_EVIDENCE",
                "baseline": "c038ba82bc09dc7d15c1734b7a9bcbf9ed65d284",
                "scope": {"root": PREFIX, "source": "git ls-files"},
                "files": sorted(files, key=lambda entry: entry["path"])}
    (PHASE / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"froze {len(files)} tracked N8-H evidence files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

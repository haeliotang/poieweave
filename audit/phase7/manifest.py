#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze tracked N7 evidence using an explicit allowlist."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
PREFIX = "audit/phase7/"
FILES = {"PREREG.md", "SEMANTICS.md", "ASSET_STATE_CONTRACT.md", "CERTIFICATE_IMPACT.md", "REPORT.zh-CN.md", "EVIDENCE.md"}
DIRS = {"canonical/", "observations/", "noninterference/", "restoration/", "regressions/", "negative-controls/", "differences/"}
EXCLUDE = {"manifest.json", "execution-manifest.json"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def allowed(relative: str) -> bool:
    if relative in FILES:
        return True
    if Path(relative).name in EXCLUDE:
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
    manifest = {
        "schema": "poie.s5v-n7-evidence/1",
        "kind": "FROZEN_TRACKED_EVIDENCE",
        "baseline": "117ba8169995c83a0d4225925e3faf6025913a9e",
        "scope": {"root": PREFIX, "source": "git ls-files", "allowlist_files": sorted(FILES), "allowlist_dirs": sorted(DIRS)},
        "files": sorted(files, key=lambda entry: entry["path"]),
    }
    (PHASE / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"froze {len(files)} tracked N7 evidence files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

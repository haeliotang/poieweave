#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze tracked N6 evidence from the Git index.

Only files tracked by Git under the explicit N6 allowlist are eligible. Runtime
caches, Python bytecode, editor files and generated execution manifests are not
part of the frozen evidence.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
PREFIX = "audit/phase6/"
ALLOWLIST = {
    "PREREG.md", "SEMANTICS.md", "RESOURCE_LIFETIME.md", "EVIDENCE.md",
    "REPORT.zh-CN.md", "config.json",
}
ALLOWED_DIRS = {
    "canonical/", "external/", "contracts/", "differences/", "negative-controls/",
    "restoration/", "scheduling/", "dependencies/", "event-boundaries/", "results/",
}
EXCLUDE_NAMES = {"manifest.json", "execution-manifest.json"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tracked() -> list[str]:
    result = subprocess.run(["git", "ls-files", "-z", PREFIX], cwd=ROOT, check=True, capture_output=True)
    return [entry.decode() for entry in result.stdout.split(b"\0") if entry]


def allowed(relative: str) -> bool:
    if relative in ALLOWLIST:
        return True
    if Path(relative).name in EXCLUDE_NAMES:
        return False
    return any(relative.startswith(directory) for directory in ALLOWED_DIRS)


def main() -> int:
    files = []
    for path in sorted(tracked()):
        relative = path[len(PREFIX):]
        if not allowed(relative):
            continue
        absolute = ROOT / path
        if not absolute.is_file():
            raise SystemExit(f"tracked evidence missing from checkout: {path}")
        files.append({"path": path, "sha256": sha(absolute), "bytes": absolute.stat().st_size})
    manifest = {
        "schema": "poie.s5v-n6-evidence/1",
        "kind": "FROZEN_TRACKED_EVIDENCE",
        "baseline": "c38ea8500426a5bc9ea9e4b521a9c4fcc0bf9423",
        "scope": {
            "root": PREFIX,
            "allowlist_files": sorted(ALLOWLIST),
            "allowlist_dirs": sorted(ALLOWED_DIRS),
            "source": "git ls-files",
            "excludes": sorted(EXCLUDE_NAMES) + ["__pycache__", "*.pyc", "*.pyo", ".DS_Store", "build/"],
        },
        "files": files,
        "tools": {
            "external_fmu": "external/fmu-n5-scheduler/poie-n5-scheduler.fmu",
            "external_fmu_sha256": sha(ROOT / "external/fmu-n5-scheduler/poie-n5-scheduler.fmu"),
            "n6_fmu": "external/fmu-n6-scheduler/poie-n6-scheduler.fmu",
            "n6_fmu_sha256": sha(ROOT / "external/fmu-n6-scheduler/poie-n6-scheduler.fmu"),
            "n6_model_source_sha256": sha(ROOT / "external/fmu-n6-scheduler/src/model.c"),
            "canonical_runner": "poie-s5v-n6-canonical",
        },
    }
    (PHASE / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"froze {len(files)} tracked N6 evidence files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

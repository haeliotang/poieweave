#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build the N4 evidence manifest with file and binary hashes."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
EXCLUDE = {"manifest.json"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    files = []
    tracked = subprocess.run(["git", "ls-files", "-z", "audit/phase4"], cwd=ROOT, check=True,
                             capture_output=True).stdout.split(b"\0")
    tracked_paths = {ROOT / entry.decode() for entry in tracked if entry}
    for path in sorted(tracked_paths):
        if not path.is_file() or path.name in EXCLUDE or "__pycache__" in path.parts or path.suffix in (".pyc", ".pyo"):
            continue
        files.append({"path": str(path.relative_to(ROOT)), "sha256": sha(path), "bytes": path.stat().st_size})
    tools = {
        "fmpy": next((line.split("=", 1)[1].strip() for line in (PHASE / "external/N4-01-fmpy-900.log").read_text().splitlines() if line.startswith("fmpy_version=")), "unknown"),
        "canonical_runner": "poie-s5v-n4-canonical",
        "external_fmu": "external/fmu-n4-pv-battery/poie-n4-pv-battery.fmu",
        "external_fmu_sha256": sha(ROOT / "external/fmu-n4-pv-battery/poie-n4-pv-battery.fmu"),
        "harness_source_sha256": sha(ROOT / "external/fmu-n4-pv-battery/src/harness.c"),
        "model_source_sha256": sha(ROOT / "external/fmu-n4-pv-battery/src/model.c"),
        "repair_fmu": "external/fmu-n4-repair/poie-n4-repair.fmu",
        "repair_fmu_sha256": sha(ROOT / "external/fmu-n4-repair/poie-n4-repair.fmu"),
        "repair_model_source_sha256": sha(ROOT / "external/fmu-n4-repair/src/model.c"),
    }
    manifest = {"schema": "poie.s5v-n4-evidence/1", "baseline": "33583f53ac046a435b6756b4a448bdc9b3ff23d9",
                "files": files, "tools": tools}
    (PHASE / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote N4 manifest with {len(files)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

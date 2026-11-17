#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build the N5 evidence manifest."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    files = []
    for path in sorted(PHASE.rglob("*")):
        if path.is_file() and path.name != "manifest.json":
            files.append({"path": str(path.relative_to(ROOT)), "sha256": sha(path), "bytes": path.stat().st_size})
    tools = {
        "external_fmu": "external/fmu-n5-scheduler/poie-n5-scheduler.fmu",
        "external_fmu_sha256": sha(ROOT / "external/fmu-n5-scheduler/poie-n5-scheduler.fmu"),
        "model_source_sha256": sha(ROOT / "external/fmu-n5-scheduler/src/model.c"),
        "harness_source_sha256": sha(ROOT / "external/fmu-n5-scheduler/src/harness.c"),
        "fmpy": next((line.split("=", 1)[1].strip() for line in
                      (PHASE / "external/N5-00-fmpy.log").read_text().splitlines()
                      if line.startswith("fmpy_version=")), "unknown"),
        "canonical_runner": "poie-s5v-n5-canonical",
    }
    (PHASE / "manifest.json").write_text(json.dumps({
        "schema": "poie.s5v-n5-evidence/1",
        "baseline": "41728951ef7e2f1e5beb6482f2e402b725bd29cb",
        "files": files,
        "tools": tools,
    }, indent=2) + "\n")
    print(f"wrote N5 manifest with {len(files)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

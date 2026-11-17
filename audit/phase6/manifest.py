#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build N6 evidence manifest."""

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
    fmu = "external/fmu-n5-scheduler/poie-n5-scheduler.fmu"
    n6fmu = "external/fmu-n6-scheduler/poie-n6-scheduler.fmu"
    (PHASE / "manifest.json").write_text(json.dumps({
        "schema": "poie.s5v-n6-evidence/1",
        "baseline": "c38ea8500426a5bc9ea9e4b521a9c4fcc0bf9423",
        "files": files,
        "tools": {"external_fmu": fmu, "external_fmu_sha256": sha(ROOT / fmu),
                  "n6_fmu": n6fmu, "n6_fmu_sha256": sha(ROOT / n6fmu),
                  "n6_model_source_sha256": sha(ROOT / "external/fmu-n6-scheduler/src/model.c"),
                  "canonical_runner": "poie-s5v-n6-canonical"},
    }, indent=2) + "\n")
    print(f"wrote N6 manifest with {len(files)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

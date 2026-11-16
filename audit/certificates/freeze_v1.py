#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze the v1 certificate bytes and runtime source environment."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent / "v1-frozen.json"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    files = []
    patterns = [
        "experiments/certificate-canonical-c*/**",
        "experiments/certificate-s2-canonical-*/**",
    ]
    for pattern in patterns:
        for path in sorted(ROOT.glob(pattern)):
            if path.is_file():
                files.append({"path": str(path.relative_to(ROOT)), "sha256": sha(path)})
    runtime = [
        "crates/poie-viability/src/engine.rs",
        "crates/poie-viability/src/survival.rs",
        "crates/poie-viability/src/clock.rs",
        "crates/poie-viability/src/closure.rs",
        "crates/poie-viability/src/scheduler.rs",
        "Cargo.lock",
    ]
    baseline = {
        "schema": "poie.s5v-v1-certificate-freeze/1",
        "baseline_commit": "9f343b55c8ec47ed8c2d8d2954d60b6e51c62fc2",
        "note": "Frozen bytes of the 36 claim certificates and 5 canonical S2 certificates before Native Observatory / Certificate v2 migration. These files must not be modified; v2 must live in new directories with explicit successor links.",
        "certificate_files": files,
        "runtime_sources": [{"path": path, "sha256": sha(ROOT / path)} for path in runtime],
    }
    OUT.write_text(json.dumps(baseline, indent=2, sort_keys=True) + "\n")
    print(f"froze {len(files)} certificate files and {len(runtime)} runtime sources")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

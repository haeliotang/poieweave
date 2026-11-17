#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze the v2 certificate bytes and historical runtime source binding."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent / "v2-frozen.json"
BASELINE = "117ba8169995c83a0d4225925e3faf6025913a9e"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tracked(prefix: str) -> list[str]:
    result = subprocess.run(["git", "ls-files", "-z", prefix], cwd=ROOT, capture_output=True, check=True)
    return [entry.decode() for entry in result.stdout.split(b"\0") if entry]


def main() -> int:
    certificate_files = []
    for path in sorted(tracked("experiments")):
        if "/certificate-canonical-" in path and path.split("/")[1].endswith("-v2"):
            certificate_files.append({"path": path, "sha256": sha(ROOT / path)})
    v2_index = "docs/research/canonical-claim-certificate-index-v2.json"
    runtime = [
        "crates/poie-viability/src/engine.rs", "crates/poie-viability/src/survival.rs",
        "crates/poie-viability/src/clock.rs", "crates/poie-viability/src/closure.rs",
        "crates/poie-viability/src/scheduler.rs", "Cargo.toml", "Cargo.lock",
    ]
    data = {
        "schema": "poie.s5v-v2-certificate-freeze/1",
        "baseline_commit": BASELINE,
        "note": "Frozen v2 certificate bytes and runtime environment before native asset-state observatory / v3 migration.",
        "v2_index": {"path": v2_index, "sha256": sha(ROOT / v2_index)},
        "certificate_files": certificate_files,
        "runtime_sources": [{"path": path, "sha256": sha(ROOT / path)} for path in runtime],
    }
    OUT.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(f"froze {len(certificate_files)} v2 certificate files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze v4 certificate bytes and runtime binding before N9 resource observation changes."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent / "v4-frozen.json"
BASELINE = "8e4b7c2164d58e8294ddeb635ab2f08f47d68377"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tracked(prefix: str) -> list[str]:
    result = subprocess.run(["git", "ls-files", "-z", prefix], cwd=ROOT, check=True, capture_output=True)
    return [entry.decode() for entry in result.stdout.split(b"\0") if entry]


def main() -> int:
    files = []
    for path in sorted(tracked("experiments")):
        if "/certificate-canonical-" in path and path.split("/")[1].endswith("-v4"):
            files.append({"path": path, "sha256": sha(ROOT / path)})
    runtime = ["crates/poie-viability/src/engine.rs", "crates/poie-viability/src/survival.rs",
               "crates/poie-viability/src/clock.rs", "crates/poie-viability/src/closure.rs",
               "crates/poie-viability/src/scheduler.rs", "Cargo.toml", "Cargo.lock"]
    data = {"schema": "poie.s5v-v4-certificate-freeze/1", "baseline_commit": BASELINE,
            "v4_index": {"path": "docs/research/canonical-claim-certificate-index-v4.json",
                         "sha256": sha(ROOT / "docs/research/canonical-claim-certificate-index-v4.json")},
            "certificate_files": files,
            "runtime_sources": [{"path": path, "sha256": sha(ROOT / path)} for path in runtime]}
    OUT.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(f"froze {len(files)} v4 certificate files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

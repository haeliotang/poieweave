#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Write an execution-evidence manifest for one actual run.

This is distinct from the frozen tracked-evidence manifest. It records the
environment and the current outputs of a real run and is never used to replace
frozen evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]
SOURCES = [
    "audit/phase6/canonical", "audit/phase6/external", "audit/phase6/contracts",
    "audit/phase6/differences", "audit/phase6/negative-controls", "audit/phase6/restoration",
]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    files = []
    for source in SOURCES:
        for path in sorted((ROOT / source).rglob("*")):
            if path.is_file():
                files.append({"path": str(path.relative_to(ROOT)), "sha256": sha(path), "bytes": path.stat().st_size})
    manifest = {
        "schema": "poie.s5v-n6-execution-evidence/1",
        "kind": "CURRENT_RUN_EXECUTION_EVIDENCE",
        "git_head": head,
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "fmpy_log": next((line.split("=", 1)[1].strip() for line in
                          (PHASE / "external/N6-B00-fmpy.log").read_text().splitlines()
                          if line.startswith("fmpy_version=")), "unknown"),
        "files": files,
    }
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote execution manifest with {len(files)} files to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

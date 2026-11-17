#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Write the N8 current-run execution manifest outside the repository."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FILES = [
    "audit/phase8/event-observations/events.csv",
    "audit/phase8/event-observations/summary.csv",
    "audit/phase8/external/N8-01-fmpy-900.csv",
    "audit/phase8/external/N8-02-fmpy-1000.csv",
    "audit/phase8/external/N8-04-fmpy-900.csv",
    "audit/phase8/trajectory-comparisons/trajectories.json",
    "docs/research/canonical-claim-certificate-index-v4.json",
]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    data = {"schema": "poie.s5v-n8-execution-evidence/1", "kind": "CURRENT_RUN_EXECUTION_EVIDENCE",
            "git_head": head, "python": sys.version.split()[0], "platform": platform.platform(),
            "files": [{"path": path, "sha256": sha(ROOT / path)} for path in FILES]}
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    print(f"wrote N8 execution manifest to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

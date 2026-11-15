#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run the cross-process crash-tail test."""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
result = subprocess.run(
    ["cargo", "test", "-q", "-p", "poie-state", "--test", "multiprocess_crash_tail"],
    cwd=ROOT,
    capture_output=True,
    text=True,
)
if result.returncode != 0:
    print(result.stdout, result.stderr, file=sys.stderr)
    sys.exit(1)
print("multiprocess-crash=ok")

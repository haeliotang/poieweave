#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Validate N7 native observation, noninterference and v3 certificate evidence."""

from __future__ import annotations

import csv
import json
import subprocess
import sys
from pathlib import Path

PHASE = Path(__file__).resolve().parent
ROOT = PHASE.parents[1]


def validate_observations(rows: list[dict]) -> list[str]:
    problems = []
    for row in rows:
        if row["status"] == "OBSERVED" and row["status_source"] != "NATIVE_CANONICAL_RUNTIME":
            problems.append("NON_NATIVE_SOURCE")
    key = {(row["case"], row["at_s"], row["phase"], row["asset_id"]): row for row in rows}
    before = key.get(("N7-00", "10800.000", "Before", "pump-p1"))
    after = key.get(("N7-00", "10800.000", "After", "pump-p1"))
    if not before or before["asset_status"] != "Degraded":
        problems.append("EARLY_RESTORATION")
    if not after or after["asset_status"] != "Available":
        problems.append("MISSING_RESTORATION")
    interrupted = key.get(("N7-03", "7200.000", "After", "robot-m1"))
    if not interrupted or interrupted["asset_status"] != "Failed":
        problems.append("MISSING_FAILURE_STATE")
    pump_interrupted = key.get(("N7-03", "7200.000", "After", "pump-p1"))
    if not pump_interrupted or pump_interrupted["asset_status"] != "Degraded":
        problems.append("WRONG_TARGET_STATE")
    post_batch = key.get(("N7-07", "10800.001", "After", "robot-m1"))
    if not post_batch or post_batch["asset_status"] != "Failed":
        problems.append("SAME_TIME_POST_BATCH_LOST")
    horizon_failure = key.get(("N7-09", "10800.000", "After", "robot-m1"))
    if not horizon_failure or horizon_failure["asset_status"] != "Available":
        problems.append("HORIZON_FAILURE_PROCESSED")
    unknown = key.get(("N7-12", "0.000", "After", "lathe-1"))
    if not unknown or unknown["asset_status"] != "Unknown":
        problems.append("UNKNOWN_CONFUSED")
    tool = key.get(("N7-05", "9000.000", "After", "tool-t1"))
    if not tool or tool["asset_status"] != "Failed":
        problems.append("REQUIRED_TOOL_STATE")
    tool_target = key.get(("N7-05", "9000.000", "After", "a-lathe"))
    if not tool_target or tool_target["asset_status"] != "Failed":
        problems.append("TOOL_TARGET_RESTORED_ERROR")
    bus = key.get(("N7-06", "3600.000", "After", "bus-1"))
    if not bus or bus["asset_status"] != "Failed":
        problems.append("UPSTREAM_STATE_LOST")
    unobservable = [row for row in rows if row["case"] == "N7-03" and row["at_s"] == "20000.000"]
    if not unobservable or any(row["status"] != "UNOBSERVABLE_AFTER_TERMINATION" or row["asset_id"] != "*"
                               for row in unobservable):
        problems.append("POST_TERMINATION_BACKFILL")
    return problems


def main() -> int:
    problems = []
    manifest = json.loads((PHASE / "manifest.json").read_text())
    if manifest.get("kind") != "FROZEN_TRACKED_EVIDENCE":
        problems.append("manifest kind")
    import hashlib
    for entry in manifest["files"]:
        path = ROOT / entry["path"]
        if not path.is_file():
            problems.append(f"missing {entry['path']}")
        elif hashlib.sha256(path.read_bytes()).hexdigest() != entry["sha256"]:
            problems.append(f"hash {entry['path']}")
    rows = list(csv.DictReader((PHASE / "observations/asset-statuses.csv").open(), delimiter="\t"))
    if not rows:
        problems.append("empty observations")
    problems.extend(validate_observations(rows))
    derived = json.loads((PHASE / "differences/native-vs-derived.json").read_text())
    if derived.get("status") != "MATCH":
        problems.append("native-vs-derived MATCH")
    v3 = json.loads((ROOT / "docs/research/canonical-claim-certificate-index-v3.json").read_text())
    if len(v3["certificates"]) != 41:
        problems.append("v3 certificate count")
    for entry in v3["certificates"]:
        cert = json.loads((ROOT / entry["certificate"]).read_text())
        if cert.get("predecessor", {}).get("relation") != "OBSERVATION_ONLY_SUCCESSOR":
            problems.append("v3 predecessor relation")
    v2 = json.loads((ROOT / "audit/certificates/v2-frozen.json").read_text())
    if len(v2["certificate_files"]) != 82:
        problems.append("v2 frozen certificate files")
    if not (ROOT / "audit/certificates/v1-frozen.json").is_file():
        problems.append("v1 freeze")
    for command in (["python3", "scripts/verify-canonical-claim-certificates-v3.py"],
                    ["python3", "audit/certificates/verify_v1.py"]):
        if subprocess.run(command, cwd=ROOT, capture_output=True).returncode != 0:
            problems.append("verification failed: " + " ".join(command))
    if problems:
        print("N7 evidence FAIL: " + ", ".join(problems), file=sys.stderr)
        return 1
    print(f"ok N7 native observations, noninterference and v3 lineage: {len(rows)} rows, 41 v3 certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

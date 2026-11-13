#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Render a one-page Markdown summary of the S1-S4 certificate bundle."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def key_result(layer: str, certificate: dict) -> str:
    if layer == "S1":
        return f"{certificate.get('outcome')} {certificate.get('target')} ({certificate.get('total_duration_s')}s)"
    if layer == "S2":
        rows = certificate.get("rows", [])
        return f"{len(rows)} load rows; 0-2 kW material-limited, 3-4 kW energy-limited"
    if layer == "S2-joint":
        return f"independent {certificate.get('independent_recoverable')} vs joint {certificate.get('joint_recoverable')}; gap {certificate.get('optimistic_gap')}"
    if layer == "S2-regen":
        rows = certificate.get("rows", [])
        return " → ".join(f"{row.get('survived_days')}d" for row in rows)
    if layer == "S3":
        return f"{len(certificate.get('receipts', []))} receipts; approved 8 h lease"
    if layer == "S4":
        return f"{certificate.get('differential_summary')}; {certificate.get('plans_summary')}"
    return ""


def main(bundle_path: str) -> int:
    bundle = json.loads(Path(bundle_path).read_text())
    lines = [
        "# Viability Certificate Bundle — Summary",
        "",
        f"- Bundle schema: `{bundle.get('schema')}`",
        f"- Claim scope: {bundle.get('claim_scope')}",
        "",
        "| Layer | Schema | SHA-256 (prefix) | Key result |",
        "| --- | --- | --- | --- |",
    ]
    for row in bundle.get("certificates", []):
        certificate_path = ROOT / row["path"]
        certificate = json.loads(certificate_path.read_text())
        lines.append(
            f"| {row['layer']} | `{row['schema']}` | `{row['sha256'][:16]}` | {key_result(row['layer'], certificate)} |"
        )
    lines += [
        "",
        "> Model-level and isolated-simulation evidence only. This summary is not a",
        "> physical safety, lifetime, authority or industrial-autonomy certificate.",
        "",
    ]
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: render.py <bundle.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))

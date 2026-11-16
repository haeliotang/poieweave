#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Split certificate-s2-metrics into independently migratable metric groups."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CERT = ROOT / "experiments/certificate-s2-metrics/certificate.json"
OUT_JSON = ROOT / "docs/research/certificate-s2-metrics-split.json"
OUT_MD = ROOT / "docs/research/certificate-s2-metrics-split.zh-CN.md"

GROUPS = {
    "adversarial": {
        "claims": ["C11"],
        "canonical": ["s2-canonical-c11"],
        "disposition": "MIGRATED_WITH_SEMANTIC_CORRECTION",
        "reason": "minimal set filter-1; day-0 exact 0.125 d and day-10 legacy row invalid",
    },
    "adversarial_resupply": {
        "claims": ["C11"],
        "canonical": ["s2-canonical-c11"],
        "disposition": "MIGRATED_WITH_SEMANTIC_CORRECTION",
        "reason": "same C11 adapter; resupply does not close the capability gap",
    },
    "base05": {
        "claims": ["C5"],
        "canonical": ["s2-canonical-c5", "s2-canonical-c6"],
        "disposition": "MIGRATED",
        "reason": "Base-05 pump intervals and coverage agree",
    },
    "base08": {
        "claims": ["C7"],
        "canonical": ["s2-canonical-c7"],
        "disposition": "MIGRATED",
        "reason": "Base-08 regeneration 40->60 and 50% yield validated",
    },
    "energy_interventions": {
        "claims": ["C8", "C13"],
        "canonical": ["s2-canonical-c8", "s2-canonical-c13"],
        "disposition": "PARTIAL",
        "reason": "load rows agree; recharge intervention rows not canonicalised",
    },
    "insert_sharpening": {
        "claims": ["C7"],
        "canonical": ["s2-canonical-c7"],
        "disposition": "MIGRATED",
        "reason": "2/3 inserts 40/50->60, 4 already 60",
    },
    "recharge_boundary": {
        "claims": [],
        "canonical": [],
        "disposition": "NOT_MIGRATED",
        "reason": "average-recharge threshold not yet canonicalised",
    },
    "recovery_coverage": {
        "claims": [],
        "canonical": [],
        "disposition": "NOT_MIGRATED",
        "reason": "Base-05 5/10 independent coverage lacks a canonical coverage adapter",
    },
    "redundancy": {
        "claims": ["C9", "C14"],
        "canonical": ["s2-canonical-c9", "s2-canonical-c14"],
        "disposition": "PARTIAL",
        "reason": "redundant pump agrees; filter/robot restorer rows not canonicalised",
    },
    "redundancy_time": {
        "claims": ["C14"],
        "canonical": ["s2-canonical-c14"],
        "disposition": "SUPERSEDED_BASELINE",
        "reason": "redundant 90 agrees; baseline 31->2 d under canonical in-repair failures",
    },
    "regenerative_capacity_days": {
        "claims": [],
        "canonical": [],
        "disposition": "NOT_MIGRATED",
        "reason": "single scalar without scenario binding",
    },
    "resupply": {
        "claims": ["C10"],
        "canonical": ["s2-canonical-c10"],
        "disposition": "MIGRATED",
        "reason": "interval->horizon mapping agrees 4/4",
    },
    "yield_sensitivity": {
        "claims": [],
        "canonical": [],
        "disposition": "NOT_MIGRATED",
        "reason": "yield rows not canonicalised",
    },
}


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def main(argv: list[str]) -> int:
    check = "--check" in argv
    certificate = json.loads(CERT.read_text(encoding="utf-8"))
    groups = []
    for name, entry in GROUPS.items():
        value = certificate.get(name)
        if value is None:
            raise SystemExit(f"missing certificate group {name}")
        groups.append({"group": name, "values": value, **entry})
    summary = {
        "groups": len(groups),
        "migrated": sum(1 for group in groups if group["disposition"] == "MIGRATED"),
        "partial": sum(1 for group in groups if group["disposition"] == "PARTIAL"),
        "migrated_with_semantic_correction": sum(
            1 for group in groups if group["disposition"] == "MIGRATED_WITH_SEMANTIC_CORRECTION"
        ),
        "superseded_baseline": sum(1 for group in groups if group["disposition"] == "SUPERSEDED_BASELINE"),
        "not_migrated": sum(1 for group in groups if group["disposition"] == "NOT_MIGRATED"),
        "reissued": 0,
    }
    result = {"schema": "poie.metrics-certificate-split/1", "groups": groups, "summary": summary}
    rendered_json = json.dumps(result, indent=2, sort_keys=True) + "\n"
    lines = [
        "# certificate-s2-metrics Split",
        "",
        "This splits the aggregate metrics certificate into metric groups. It does not reissue it.",
        "",
        "| group | claims | canonical | disposition |",
        "| --- | --- | --- | --- |",
    ]
    for group in groups:
        lines.append(
            f"| `{group['group']}` | {', '.join(group['claims']) or 'NONE'} | "
            f"{', '.join(group['canonical']) or 'NONE'} | `{group['disposition']}` |"
        )
    lines += [
        "",
        "## Summary",
        "",
        f"- groups: {summary['groups']}",
        f"- migrated: {summary['migrated']}",
        f"- migrated with semantic correction: {summary['migrated_with_semantic_correction']}",
        f"- partial: {summary['partial']}",
        f"- superseded baseline: {summary['superseded_baseline']}",
        f"- not migrated: {summary['not_migrated']}",
        f"- reissued: {summary['reissued']}",
        "",
        "## Reasons",
        "",
    ]
    for group in groups:
        lines += [f"### `{group['group']}`", "", group["reason"], ""]
    rendered_md = "\n".join(lines) + "\n"
    if check:
        if read(OUT_JSON) != rendered_json or read(OUT_MD) != rendered_md:
            print("metrics split is stale; run scripts/build-metrics-split.py", file=sys.stderr)
            return 1
        print("metrics split is up to date")
        return 0
    OUT_JSON.write_text(rendered_json, encoding="utf-8")
    OUT_MD.write_text(rendered_md, encoding="utf-8")
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

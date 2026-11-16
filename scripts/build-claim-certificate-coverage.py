#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Claim -> canonical experiment -> certificate coverage crosswalk."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs/research/s2-evidence-inventory.json"
MATRIX = ROOT / "docs/research/certificate-impact-matrix.json"
CANONICAL_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v2.json"
OUT_JSON = ROOT / "docs/research/s2-claim-certificate-coverage.json"
OUT_MD = ROOT / "docs/research/s2-claim-certificate-coverage.zh-CN.md"

# Historical S2 claims whose canonical successor carries the canonical experiment.
SUPERSEDED_BY_CLAIM = {
    "C5": "C56",
    "C6": "C57",
    "C7": "C58",
    "C8": "C59",
    "C9": "C60",
    "C10": "C61",
    "C11": "C62",
    "C12": "C63",
    "C13": "C64",
    "C14": "C65",
    "C18": "C66",
    "C19": "C67",
    "C68": "C73",
}


def build() -> dict:
    inventory = json.loads(INVENTORY.read_text())
    matrix = json.loads(MATRIX.read_text())
    by_claim: dict[str, dict] = {}
    for cert in matrix["certificates"]:
        for claim in cert["claims"]:
            by_claim[claim] = {
                "certificate_status": cert["certificate_status"],
                "id": cert["id"],
                "canonical_certificate": cert["canonical_certificate"],
                "scope": cert.get("scope"),
            }
    if CANONICAL_INDEX.is_file():
        canonical_index = json.loads(CANONICAL_INDEX.read_text())
        for entry in canonical_index["certificates"]:
            by_claim[entry["claim"]] = {
                "certificate_status": entry["status"],
                "id": f"canonical-{entry['claim'].lower()}",
                "canonical_certificate": entry["certificate"],
                "scope": entry["classification"],
            }
    rows = []
    for claim in inventory["claims"]:
        cid = claim["id"]
        replacement = SUPERSEDED_BY_CLAIM.get(cid)
        canonical_claim = replacement or cid
        canonical = [
            experiment["experiment"]
            for experiment in inventory["experiments"]
            if canonical_claim in experiment["claim_ids"] and experiment["experiment"].startswith("s2-canonical")
        ]
        cert = by_claim.get(cid)
        if cert is None:
            status = "NOT_CERTIFIED"
            certificate_id = None
            canonical_certificate = None
            scope = None
        else:
            status = cert["certificate_status"]
            certificate_id = cert["id"]
            canonical_certificate = cert["canonical_certificate"]
            scope = cert.get("scope")
        if cert is None and replacement is not None:
            status = "SUPERSEDED_BY_CLAIM"
            certificate_id = None
            canonical_certificate = None
            scope = None
        rows.append(
            {
                "claim": cid,
                "superseded_by": replacement,
                "statement": claim["statement"],
                "canonical_experiments": canonical,
                "certificate_status": status,
                "certificate_id": certificate_id,
                "canonical_certificate": canonical_certificate,
                "scope": scope,
            }
        )
    summary = {
        "claims": len(rows),
        "claims_superseded_by_claim": len(SUPERSEDED_BY_CLAIM),
        "canonical_experiments": sum(1 for row in rows if row["canonical_experiments"]),
        "certificate_statuses": {
            status: sum(1 for row in rows if row["certificate_status"] == status)
            for status in sorted({row["certificate_status"] for row in rows})
        },
    }
    return {"schema": "poie.claim-certificate-coverage/1", "claims": rows, "summary": summary}


def render(data: dict) -> str:
    lines = [
        "# S2 claim → canonical experiment → certificate 覆盖",
        "",
        "> 机器生成。`NOT_CERTIFIED` 是显式缺口，不是已认证声明。",
        "",
        "## 汇总",
        "",
        "```text",
        f"claims              {data['summary']['claims']}",
        f"有 canonical 实验   {data['summary']['canonical_experiments']}",
        f"旧 claim 被新 claim 取代 {data['summary']['claims_superseded_by_claim']}",
    ]
    for status, count in data["summary"]["certificate_statuses"].items():
        lines.append(f"{status:<19} {count}")
    lines += ["```", "", "## 逐项", "", "| claim | canonical experiment | certificate | status |", "| --- | --- | --- | --- |"]
    for row in data["claims"]:
        exp = ", ".join(row["canonical_experiments"]) or "—"
        cert = row["certificate_id"] or "—"
        shown = f"{row['claim']} → {row['superseded_by']}" if row.get("superseded_by") else row["claim"]
        lines.append(f"| {shown} | {exp} | {cert} | {row['certificate_status']} |")
    lines += [
        "",
        "## 解释",
        "",
        "- `REISSUED`：已有 canonical 证书。",
        "- `SUPERSEDED`：历史证书有效于原始语义，但已被 canonical 语义取代。",
        "- `NOT_CERTIFIED`：有 canonical 实验，但尚未签发新证书；不得当作已认证。",
        "- `SUPERSEDED_BY_CLAIM`：历史 claim 已由带 canonical 实验的新 claim 取代；新 claim 的证书状态见对应行。",
        "- 本表不改变既有证书，也不把辅助覆盖自动升级为正式认证。",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    data = build()
    rendered = render(data)
    if "--check" in argv:
        current = json.loads(OUT_JSON.read_text()) if OUT_JSON.exists() else None
        if current != data:
            print("claim certificate coverage is stale; run scripts/build-claim-certificate-coverage.py")
            return 1
        if OUT_MD.read_text() != rendered:
            print("claim certificate coverage markdown is stale; run the generator")
            return 1
        print(f"ok claim certificate coverage: {data['summary']['claims']} claims")
        return 0
    OUT_JSON.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    OUT_MD.write_text(rendered)
    print(json.dumps(data["summary"], ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

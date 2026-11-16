#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build the S2 certificate impact matrix from certificate bundles and inventory."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CERTS = ROOT / "experiments"
INVENTORY = ROOT / "docs/research/s2-evidence-inventory.json"
OUT_JSON = ROOT / "docs/research/certificate-impact-matrix.json"
OUT_MD = ROOT / "docs/research/certificate-impact-matrix.zh-CN.md"

# Deliberate reviewer mapping. A certificate is not "reissued" just because a
# canonical experiment agrees; reissue is a separate Phase D action.
MAPPING = {
    "certificate-s2-time": {
        "claims": ["C12", "C13", "C18", "C19"],
        "canonical": ["s2-canonical-c12", "s2-canonical-c13", "s2-canonical-c18", "s2-canonical-c19"],
        "status": "SUPERSEDED",
        "classification": "SEMANTIC_CORRECTION",
        "scope": "time semantics and boundaries",
        "rationale": "canonical t=0 ordering and in-repair failure timing change grease/random-window values",
    },
    "certificate-s2-joint": {
        "claims": ["C6"],
        "canonical": ["s2-canonical-c6"],
        "status": "HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS",
        "classification": "EXPECTED_AGREEMENT",
        "scope": "Base-06 joint coverage",
        "rationale": "canonical reproduces 25/56 independent and 24/56 joint after the non-service repair fix",
    },
    "certificate-s2-load": {
        "claims": ["C69"],
        "canonical": ["s2-canonical-load-base05"],
        "status": "HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS",
        "classification": "EXPECTED_AGREEMENT",
        "scope": "Base-05 load 0-4 kW",
        "rationale": "canonical reproduces all five Base-05 load rows with final energy and termination",
    },
    "certificate-s2-regeneration": {
        "claims": ["C70"],
        "canonical": ["s2-canonical-regen-horizon"],
        "status": "HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS",
        "classification": "EXPECTED_AGREEMENT",
        "scope": "Base-05/08 regeneration intervals",
        "rationale": "canonical reproduces 8/8 rows including final energy and termination",
    },
    "certificate-s2-metrics": {
        "claims": [],
        "canonical": [],
        "status": "BLOCKED",
        "classification": "NOT_MIGRATED",
        "scope": "cross-model metric definitions and values",
        "rationale": "split into 13 metric groups: 4 migrated, 2 semantic, 2 partial, 1 superseded baseline, 4 not migrated; no reissue",
    },
    "certificate-s2-order": {
        "claims": [],
        "canonical": [],
        "status": "BLOCKED",
        "classification": "NOT_MIGRATED",
        "scope": "Base-06 repair-order sensitivity",
        "rationale": "no formal claim in the claims ledger and no canonical order adapter; do not reissue",
    },
}


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def main(argv: list[str]) -> int:
    check = "--check" in argv
    certs = []
    for name, entry in MAPPING.items():
        directory = CERTS / name
        certificate = json.loads((directory / "certificate.json").read_text(encoding="utf-8"))
        expected = read(directory / "expected.txt")
        rows = {
            "id": name,
            "model": certificate.get("model", "multi-model"),
            "model_sha256": certificate.get("model_sha256"),
            "schema": certificate.get("schema"),
            "claim_scope": certificate.get("claim_scope"),
            "claims": entry["claims"],
            "canonical_experiments": entry["canonical"],
            "canonical_classification": entry["classification"],
            "certificate_status": entry["status"],
            "scope": entry["scope"],
            "rationale": entry["rationale"],
            "expected_sha256": hashlib.sha256(expected.encode("utf-8")).hexdigest() if expected else None,
            "reissued": False,
        }
        certs.append(rows)
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    summary = {
        "certificates": len(certs),
        "historical_valid": sum(1 for cert in certs if cert["certificate_status"] == "HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS"),
        "superseded": sum(1 for cert in certs if cert["certificate_status"] == "SUPERSEDED"),
        "invalidated": sum(1 for cert in certs if cert["certificate_status"] == "INVALIDATED"),
        "reissued": sum(1 for cert in certs if cert["certificate_status"] == "REISSUED"),
        "blocked": sum(1 for cert in certs if cert["certificate_status"] == "BLOCKED"),
        "canonical_migrations": inventory["summary"]["formal_s2_experiments_migrated_to_canonical"],
        "canonical_semantic_corrections": inventory["summary"]["canonical_semantic_corrections"],
    }
    result = {"schema": "poie.s2-certificate-impact-matrix/1", "certificates": certs, "summary": summary}
    rendered_json = json.dumps(result, indent=2, sort_keys=True) + "\n"
    lines = [
        "# S2 Certificate Impact Matrix",
        "",
        "This matrix records certificate impact only. No certificate is reissued by this document.",
        "",
        "| certificate | model | claims | canonical | classification | status | reissued |",
        "| --- | --- | --- | --- | --- | --- | --- |",
    ]
    for cert in certs:
        lines.append(
            f"| `{cert['id']}` | `{cert['model']}` | {', '.join(cert['claims']) or 'NONE'} | "
            f"{', '.join(cert['canonical_experiments']) or 'NONE'} | `{cert['canonical_classification']}` | "
            f"`{cert['certificate_status']}` | no |"
        )
    lines += [
        "",
        "## Summary",
        "",
        f"- certificates: {summary['certificates']}",
        f"- historical valid with original semantics: {summary['historical_valid']}",
        f"- superseded: {summary['superseded']}",
        f"- invalidated: {summary['invalidated']}",
        f"- reissued: {summary['reissued']}",
        f"- blocked: {summary['blocked']}",
        "",
        "## Rationale",
        "",
    ]
    for cert in certs:
        lines += [f"### `{cert['id']}`", "", cert["rationale"], "", f"- scope: {cert['scope']}"]
        if cert["model_sha256"]:
            lines += [f"- model sha256: `{cert['model_sha256']}`"]
        lines += [""]
    rendered_md = "\n".join(lines) + "\n"
    if check:
        if read(OUT_JSON) != rendered_json or read(OUT_MD) != rendered_md:
            print("certificate impact matrix is stale; run scripts/build-certificate-impact.py", file=sys.stderr)
            return 1
        print("certificate impact matrix is up to date")
        return 0
    OUT_JSON.write_text(rendered_json, encoding="utf-8")
    OUT_MD.write_text(rendered_md, encoding="utf-8")
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

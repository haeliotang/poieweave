#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Issue canonical evidence certificates for migrated S2 claims.

These are new evidence packages. Historical certificates are never rewritten.
Each certificate binds the claim, canonical experiment output, runner, engine,
models and the claims ledger. Claims without sufficient canonical evidence are
not listed here and therefore stay NOT_CERTIFIED.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs/research/s2-evidence-inventory.json"
INDEX = ROOT / "docs/research/canonical-claim-certificate-index.json"
CLAIMS = [
    "C56", "C57", "C58", "C59", "C60", "C61", "C62", "C63", "C64", "C65", "C66", "C67", "C73",
]
CLASSIFICATION = {
    "CANONICAL_MIGRATED": "EXPECTED_AGREEMENT",
    "CANONICAL_SEMANTIC_CORRECTION": "SEMANTIC_CORRECTION",
}
SEMANTICS = (
    "canonical event engine: monotonic EventClock; t=0 failures after initial repair; "
    "in-repair failures apply at event time; requires_any repaired only when no alternative; "
    "energy ledger conservation"
)
VERIFIER = "poie-canonical-s2-certificate/2"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_one(claim: dict, experiment: dict) -> tuple[dict, str, str]:
    claim_id = claim["id"]
    exp_path = ROOT / "experiments" / experiment["experiment"]
    models = [{"path": f"scenarios/{name}/model.toml"} for name in experiment["models"]]
    sources = [
        {"path": f"experiments/{experiment['experiment']}/expected.txt", "sha256": sha256(exp_path / "expected.txt")},
        {"path": f"experiments/{experiment['experiment']}/REPORT.md", "sha256": sha256(exp_path / "REPORT.md")},
        {"path": f"experiments/{experiment['experiment']}/PREREG.md", "sha256": sha256(exp_path / "PREREG.md")},
    ]
    runner = experiment["runner_binaries"][0]
    certificate = {
        "schema": "poie.canonical-s2-certificate/2",
        "certificate_id": f"poie-s2-canonical-{claim_id.lower()}-v1",
        "status": "CANONICAL_ISSUED",
        "claims": [claim_id],
        "claims_sha256": sha256(ROOT / "docs/research/claims-ledger.zh-CN.md"),
        "claim_statement": claim["statement"],
        "canonical_classification": CLASSIFICATION[experiment["migration_status"]],
        "semantics": SEMANTICS,
        "scope": f"model/simulation evidence only; canonical claim {claim_id}",
        "engine": {
            "source": "crates/poie-viability/src/engine.rs",
            "sha256": sha256(ROOT / "crates/poie-viability/src/engine.rs"),
        },
        "runner": {
            "binary": runner,
            "source": f"crates/poie-viability/src/bin/{runner}.rs",
            "sha256": sha256(ROOT / f"crates/poie-viability/src/bin/{runner}.rs"),
        },
        "experiment": {
            "path": f"experiments/{experiment['experiment']}",
            "expected": f"experiments/{experiment['experiment']}/expected.txt",
            "expected_sha256": sha256(exp_path / "expected.txt"),
        },
        "models": [{"path": model["path"], "sha256": sha256(ROOT / model["path"])} for model in models],
        "evidence_sources": sources,
        "verifier": VERIFIER,
    }
    readme = (
        f"# {certificate['certificate_id']}\n\n"
        "New canonical evidence certificate. No historical certificate is modified.\n\n"
        f"- claim: {claim_id}\n"
        f"- canonical experiment: `{experiment['experiment']}`\n"
        f"- classification: `{certificate['canonical_classification']}`\n"
        f"- status: `CANONICAL_ISSUED`\n"
        f"- verifier: `{VERIFIER}`\n"
    )
    return certificate, readme, claim_id


def build_all() -> tuple[dict, dict[str, tuple[dict, str]]]:
    inventory = json.loads(INVENTORY.read_text())
    claims = {claim["id"]: claim for claim in inventory["claims"]}
    experiments = inventory["experiments"]
    results: dict[str, tuple[dict, str]] = {}
    index = {"schema": "poie.canonical-claim-certificate-index/1", "certificates": []}
    for claim_id in CLAIMS:
        claim = claims[claim_id]
        experiment = next(e for e in experiments if claim_id in e["claim_ids"])
        certificate, readme, _ = build_one(claim, experiment)
        directory = f"experiments/certificate-canonical-{claim_id.lower()}"
        results[f"{directory}/certificate.json"] = (certificate, json.dumps(certificate, indent=2, sort_keys=True) + "\n")
        results[f"{directory}/README.md"] = (certificate, readme)
        index["certificates"].append(
            {
                "claim": claim_id,
                "certificate": f"{directory}/certificate.json",
                "experiment": experiment["experiment"],
                "classification": certificate["canonical_classification"],
                "status": certificate["status"],
            }
        )
    index["summary"] = {
        "certificates": len(index["certificates"]),
        "claims": sorted(entry["claim"] for entry in index["certificates"]),
    }
    return index, results


def main(argv: list[str]) -> int:
    index, results = build_all()
    index_text = json.dumps(index, indent=2, sort_keys=True) + "\n"
    stale = []
    if "--check" in argv:
        if not INDEX.is_file() or INDEX.read_text() != index_text:
            stale.append(str(INDEX.relative_to(ROOT)))
        for rel, (_, content) in results.items():
            path = ROOT / rel
            if not path.is_file() or path.read_text() != content:
                stale.append(rel)
        if stale:
            print("canonical claim certificates are stale: " + ", ".join(stale), file=sys.stderr)
            return 1
        print(f"ok canonical claim certificates: {index['summary']['certificates']}")
        return 0
    INDEX.write_text(index_text)
    for rel, (_, content) in results.items():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    print(f"wrote {len(index['certificates'])} canonical claim certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Issue v2 canonical claim certificates binding the full runtime environment.

v1 files are frozen by audit/certificates/v1-frozen.json and are never modified.
v2 lives in new directories and records an explicit predecessor.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs/research/s2-evidence-inventory.json"
V2_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v2.json"
V1_INDEX = ROOT / "docs/research/canonical-claim-certificate-index.json"

CLAIMS = [f"C{i}" for i in range(56, 68)] + ["C6", "C69", "C70", "C71", "C72", "C73"] + [f"C{i}" for i in range(74, 97)]
MANUAL = {
    "C6": ("s2-canonical-c6", "poie-migration-c6", ["base-06"], "EXPECTED_AGREEMENT",
           "experiments/certificate-s2-canonical-joint/certificate.json"),
    "C69": ("s2-canonical-load-base05", "poie-migration-s2-load-base05", ["base-05"], "EXPECTED_AGREEMENT",
            "experiments/certificate-s2-canonical-load/certificate.json"),
    "C70": ("s2-canonical-regen-horizon", "poie-migration-s2-regen-horizon", ["base-05", "base-08"], "EXPECTED_AGREEMENT",
            "experiments/certificate-s2-canonical-regeneration/certificate.json"),
    "C71": ("s2-canonical-metrics-groups", "poie-migration-metrics-groups", ["base-05", "base-06", "base-08"],
            "MIXED_AGREEMENT_AND_DEFINITION_CHANGE", "experiments/certificate-s2-canonical-metrics/certificate.json"),
    "C72": ("s2-canonical-order", "poie-migration-order", ["base-06"], "EXPECTED_AGREEMENT",
            "experiments/certificate-s2-canonical-order/certificate.json"),
}
CLASSIFICATION = {"CANONICAL_MIGRATED": "EXPECTED_AGREEMENT", "CANONICAL_SEMANTIC_CORRECTION": "SEMANTIC_CORRECTION"}
RUNTIME = ["crates/poie-viability/src/engine.rs", "crates/poie-viability/src/survival.rs",
           "crates/poie-viability/src/clock.rs", "crates/poie-viability/src/closure.rs",
           "crates/poie-viability/src/scheduler.rs", "Cargo.toml", "Cargo.lock"]
SEMANTICS = ("canonical event engine: monotonic EventClock; t=0 failures after initial repair; "
             "in-repair failures apply at event time; requires_any repaired only when no alternative; "
             "energy ledger conservation; native read-only observation does not alter execution")
VERIFIER = "poie-canonical-s2-certificate/3"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def env(paths: list[str]) -> dict:
    return {"files": [{"path": p, "sha256": sha(ROOT / p)} for p in paths]}


def main(argv: list[str]) -> int:
    inventory = json.loads(INVENTORY.read_text())
    claims = {claim["id"]: claim for claim in inventory["claims"]}
    experiments = {experiment["experiment"]: experiment for experiment in inventory["experiments"]}
    v1 = {entry["claim"]: entry for entry in json.loads(V1_INDEX.read_text())["certificates"]}
    index = {"schema": "poie.canonical-claim-certificate-index/2", "certificates": []}
    results: dict[str, str] = {}
    for claim_id in CLAIMS:
        claim = claims[claim_id]
        if claim_id in MANUAL:
            experiment_name, runner, model_names, classification, predecessor = MANUAL[claim_id]
        else:
            entry = next(e for e in experiments.values() if claim_id in e["claim_ids"])
            experiment_name = entry["experiment"]
            runner = entry["runner_binaries"][0]
            model_names = entry["models"]
            classification = CLASSIFICATION[entry["migration_status"]]
            predecessor = None
        exp_path = ROOT / "experiments" / experiment_name
        predecessor_path = None
        predecessor_sha = None
        if predecessor is None and claim_id in v1:
            predecessor_path = v1[claim_id]["certificate"]
            predecessor_sha = sha(ROOT / predecessor_path)
        elif predecessor is not None:
            predecessor_path = predecessor
            predecessor_sha = sha(ROOT / predecessor)
        paths = RUNTIME + [
            f"crates/poie-viability/src/bin/{runner}.rs",
            *[f"scenarios/{name}/model.toml" for name in model_names],
            f"experiments/{experiment_name}/expected.txt",
            f"experiments/{experiment_name}/REPORT.md",
            f"experiments/{experiment_name}/PREREG.md",
        ]
        certificate = {
            "schema": "poie.canonical-s2-certificate/3",
            "certificate_id": f"poie-s2-canonical-{claim_id.lower()}-v2",
            "version": 2,
            "status": "CANONICAL_ISSUED_V2",
            "claims": [claim_id],
            "claim_statement": claim["statement"],
            "claims_sha256": sha(ROOT / "docs/research/claims-ledger.zh-CN.md"),
            "canonical_classification": classification,
            "semantics": SEMANTICS,
            "scope": f"model/simulation evidence only; canonical claim {claim_id}; native observation does not change execution",
            "engine": {"source": "crates/poie-viability/src/engine.rs", "sha256": sha(ROOT / "crates/poie-viability/src/engine.rs")},
            "runner": {"binary": runner, "source": f"crates/poie-viability/src/bin/{runner}.rs",
                        "sha256": sha(ROOT / f"crates/poie-viability/src/bin/{runner}.rs")},
            "experiment": {"path": f"experiments/{experiment_name}",
                            "expected": f"experiments/{experiment_name}/expected.txt",
                            "expected_sha256": sha(exp_path / "expected.txt")},
            "models": [{"path": f"scenarios/{name}/model.toml", "sha256": sha(ROOT / f"scenarios/{name}/model.toml")}
                       for name in model_names],
            "source_environment": env(paths),
            "predecessor": None if predecessor_path is None else {
                "path": predecessor_path,
                "sha256": predecessor_sha,
                "relation": "v1 certificate superseded by v2 runtime-environment binding",
            },
            "verifier": VERIFIER,
        }
        directory = f"experiments/certificate-canonical-{claim_id.lower()}-v2"
        results[f"{directory}/certificate.json"] = json.dumps(certificate, indent=2, sort_keys=True) + "\n"
        results[f"{directory}/README.md"] = (
            f"# {certificate['certificate_id']}\n\n"
            "v2 canonical evidence certificate. v1 is frozen unchanged and linked as predecessor.\n\n"
            f"- claim: {claim_id}\n- canonical experiment: `{experiment_name}`\n"
            f"- classification: `{classification}`\n- status: `CANONICAL_ISSUED_V2`\n"
            f"- predecessor: `{predecessor_path}`\n"
        )
        index["certificates"].append({
            "claim": claim_id,
            "certificate": f"{directory}/certificate.json",
            "experiment": experiment_name,
            "classification": classification,
            "status": "CANONICAL_ISSUED_V2",
            "predecessor": predecessor_path,
        })
    index["summary"] = {"certificates": len(index["certificates"]), "claims": sorted(entry["claim"] for entry in index["certificates"])}
    results[str(V2_INDEX.relative_to(ROOT))] = json.dumps(index, indent=2, sort_keys=True) + "\n"
    stale = []
    if "--check" in argv:
        for rel, content in results.items():
            path = ROOT / rel
            if not path.is_file() or path.read_text() != content:
                stale.append(rel)
        if stale:
            print("v2 canonical claim certificates are stale: " + ", ".join(stale), file=sys.stderr)
            return 1
        print(f"ok v2 canonical claim certificates: {len(index['certificates'])}")
        return 0
    for rel, content in results.items():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    print(f"wrote {len(index['certificates'])} v2 certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

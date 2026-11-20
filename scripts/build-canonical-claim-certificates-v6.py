#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Issue v4 canonical claim certificates with native-observation binding.

v2 certificate bytes are frozen by audit/certificates/v2-frozen.json and are
never modified. v5 predecessors point to v3.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs/research/s2-evidence-inventory.json"
V5_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v5.json"
V6_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v6.json"
N10_MANIFEST = ROOT / "audit/phase10/manifest.json"

CLAIMS = [f"C{i}" for i in range(56, 68)] + ["C6", "C69", "C70", "C71", "C72", "C73"] + [f"C{i}" for i in range(74, 97)]
RUNTIME = ["crates/poie-viability/src/engine.rs", "crates/poie-viability/src/survival.rs",
           "crates/poie-viability/src/clock.rs", "crates/poie-viability/src/closure.rs",
           "crates/poie-viability/src/scheduler.rs", "Cargo.toml", "Cargo.lock"]
SEMANTICS = ("canonical event engine: monotonic EventClock; t=0 failures after initial repair; "
             "in-repair failures apply at event time; requires_any repaired only when no alternative; "
             "energy ledger conservation; native read-only AssetStatus, event, resource-witness and task-resource observation do not alter execution")
VERIFIER = "poie-canonical-s2-certificate/7"
REPLAY_COMMAND = ["python3", "scripts/check-experiment-replays.py"]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def env(paths: list[str]) -> dict:
    return {"files": [{"path": p, "sha256": sha(ROOT / p)} for p in paths]}


def replay() -> dict:
    result = subprocess.run(REPLAY_COMMAND, cwd=ROOT, capture_output=True, check=False)
    if result.returncode != 0:
        raise SystemExit("experiment replay failed: " + result.stdout.decode(errors="replace")[-1000:])
    output = result.stdout + result.stderr
    return {"command": "python3 scripts/check-experiment-replays.py",
            "output_sha256": hashlib.sha256(output).hexdigest(), "exit_code": result.returncode}


def main(argv: list[str]) -> int:
    inventory = json.loads(INVENTORY.read_text())
    claims = {claim["id"]: claim for claim in inventory["claims"]}
    experiments = {experiment["experiment"]: experiment for experiment in inventory["experiments"]}
    v5_index = json.loads(V5_INDEX.read_text())
    v5 = {entry["claim"]: entry for entry in v5_index["certificates"]}
    replay_evidence = replay()
    native = {"path": "audit/phase10/manifest.json", "sha256": sha(N10_MANIFEST)}
    index = {"schema": "poie.canonical-claim-certificate-index/6", "certificates": []}
    results: dict[str, str] = {}
    for claim_id in CLAIMS:
        claim = claims[claim_id]
        entry = next(e for e in experiments.values() if claim_id in e["claim_ids"])
        experiment_name = entry["experiment"]
        runner = entry["runner_binaries"][0]
        model_names = entry["models"]
        v5_entry = v5[claim_id]
        v5_path = v5_entry["certificate"]
        v5_sha = sha(ROOT / v5_path)
        paths = RUNTIME + [
            f"crates/poie-viability/src/bin/{runner}.rs",
            *[f"scenarios/{name}/model.toml" for name in model_names],
            f"experiments/{experiment_name}/expected.txt",
            f"experiments/{experiment_name}/REPORT.md",
            f"experiments/{experiment_name}/PREREG.md",
        ]
        certificate = {
            "schema": "poie.canonical-s2-certificate/7",
            "certificate_id": f"poie-s2-canonical-{claim_id.lower()}-v6",
            "version": 6,
            "status": "CANONICAL_ISSUED_V6",
            "claims": [claim_id],
            "claim_statement": claim["statement"],
            "claims_sha256": sha(ROOT / "docs/research/claims-ledger.zh-CN.md"),
            "canonical_classification": v5_entry["classification"],
            "semantics": SEMANTICS,
            "scope": f"model/simulation evidence only; canonical claim {claim_id}; native observation does not change execution",
            "engine": {"source": "crates/poie-viability/src/engine.rs",
                        "sha256": sha(ROOT / "crates/poie-viability/src/engine.rs")},
            "runner": {"binary": runner, "source": f"crates/poie-viability/src/bin/{runner}.rs",
                        "sha256": sha(ROOT / f"crates/poie-viability/src/bin/{runner}.rs")},
            "experiment": {"path": f"experiments/{experiment_name}",
                            "expected": f"experiments/{experiment_name}/expected.txt",
                            "expected_sha256": sha(ROOT / f"experiments/{experiment_name}/expected.txt")},
            "models": [{"path": f"scenarios/{name}/model.toml", "sha256": sha(ROOT / f"scenarios/{name}/model.toml")}
                       for name in model_names],
            "source_environment": env(paths),
            "native_task_resource_observatory": native,
            "replay": replay_evidence,
            "predecessor": {"path": v5_path, "sha256": v5_sha, "relation": "TASK_RESOURCE_OBSERVATION_ONLY_SUCCESSOR"},
            "verifier": VERIFIER,
        }
        directory = f"experiments/certificate-canonical-{claim_id.lower()}-v6"
        results[f"{directory}/certificate.json"] = json.dumps(certificate, indent=2, sort_keys=True) + "\n"
        results[f"{directory}/README.md"] = (
            f"# {certificate['certificate_id']}\n\n"
            "v6 canonical evidence certificate with native task-resource observatory binding. v5 bytes are frozen unchanged.\n\n"
            f"- claim: {claim_id}\n- canonical experiment: `{experiment_name}`\n"
            f"- classification: `{v5_entry['classification']}`\n- status: `CANONICAL_ISSUED_V6`\n"
            f"- predecessor: `{v5_path}`\n- native observatory: `audit/phase10/manifest.json`\n"
        )
        index["certificates"].append({
            "claim": claim_id,
            "certificate": f"{directory}/certificate.json",
            "experiment": experiment_name,
            "classification": v5_entry["classification"],
            "status": "CANONICAL_ISSUED_V6",
            "predecessor": v5_path,
        })
    index["summary"] = {"certificates": len(index["certificates"]),
                        "claims": sorted(entry["claim"] for entry in index["certificates"])}
    results[str(V6_INDEX.relative_to(ROOT))] = json.dumps(index, indent=2, sort_keys=True) + "\n"
    stale = []
    if "--check" in argv:
        for rel, content in results.items():
            path = ROOT / rel
            if not path.is_file() or path.read_text() != content:
                stale.append(rel)
        if stale:
            print("v4 canonical claim certificates are stale: " + ", ".join(stale[:5]), file=sys.stderr)
            return 1
        print(f"ok v4 canonical claim certificates: {len(index['certificates'])}")
        return 0
    for rel, content in results.items():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    print(f"wrote {len(index['certificates'])} v6 certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

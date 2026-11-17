#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Issue v3 canonical claim certificates with native-observation binding.

v2 certificate bytes are frozen by audit/certificates/v2-frozen.json and are
never modified. v3 predecessors point to v2.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs/research/s2-evidence-inventory.json"
V2_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v2.json"
V3_INDEX = ROOT / "docs/research/canonical-claim-certificate-index-v3.json"
N7_MANIFEST = ROOT / "audit/phase7/manifest.json"

CLAIMS = [f"C{i}" for i in range(56, 68)] + ["C6", "C69", "C70", "C71", "C72", "C73"] + [f"C{i}" for i in range(74, 97)]
RUNTIME = ["crates/poie-viability/src/engine.rs", "crates/poie-viability/src/survival.rs",
           "crates/poie-viability/src/clock.rs", "crates/poie-viability/src/closure.rs",
           "crates/poie-viability/src/scheduler.rs", "Cargo.toml", "Cargo.lock"]
SEMANTICS = ("canonical event engine: monotonic EventClock; t=0 failures after initial repair; "
             "in-repair failures apply at event time; requires_any repaired only when no alternative; "
             "energy ledger conservation; native read-only AssetStatus observation does not alter execution")
VERIFIER = "poie-canonical-s2-certificate/4"
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
    v2_index = json.loads(V2_INDEX.read_text())
    v2 = {entry["claim"]: entry for entry in v2_index["certificates"]}
    replay_evidence = replay()
    native = {"path": "audit/phase7/manifest.json", "sha256": sha(N7_MANIFEST)}
    index = {"schema": "poie.canonical-claim-certificate-index/3", "certificates": []}
    results: dict[str, str] = {}
    for claim_id in CLAIMS:
        claim = claims[claim_id]
        entry = next(e for e in experiments.values() if claim_id in e["claim_ids"])
        experiment_name = entry["experiment"]
        runner = entry["runner_binaries"][0]
        model_names = entry["models"]
        v2_entry = v2[claim_id]
        v2_path = v2_entry["certificate"]
        v2_sha = sha(ROOT / v2_path)
        paths = RUNTIME + [
            f"crates/poie-viability/src/bin/{runner}.rs",
            *[f"scenarios/{name}/model.toml" for name in model_names],
            f"experiments/{experiment_name}/expected.txt",
            f"experiments/{experiment_name}/REPORT.md",
            f"experiments/{experiment_name}/PREREG.md",
        ]
        certificate = {
            "schema": "poie.canonical-s2-certificate/4",
            "certificate_id": f"poie-s2-canonical-{claim_id.lower()}-v3",
            "version": 3,
            "status": "CANONICAL_ISSUED_V3",
            "claims": [claim_id],
            "claim_statement": claim["statement"],
            "claims_sha256": sha(ROOT / "docs/research/claims-ledger.zh-CN.md"),
            "canonical_classification": v2_entry["classification"],
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
            "native_observatory": native,
            "replay": replay_evidence,
            "predecessor": {"path": v2_path, "sha256": v2_sha, "relation": "OBSERVATION_ONLY_SUCCESSOR"},
            "verifier": VERIFIER,
        }
        directory = f"experiments/certificate-canonical-{claim_id.lower()}-v3"
        results[f"{directory}/certificate.json"] = json.dumps(certificate, indent=2, sort_keys=True) + "\n"
        results[f"{directory}/README.md"] = (
            f"# {certificate['certificate_id']}\n\n"
            "v3 canonical evidence certificate with native-observatory binding. v2 bytes are frozen unchanged.\n\n"
            f"- claim: {claim_id}\n- canonical experiment: `{experiment_name}`\n"
            f"- classification: `{v2_entry['classification']}`\n- status: `CANONICAL_ISSUED_V3`\n"
            f"- predecessor: `{v2_path}`\n- native observatory: `audit/phase7/manifest.json`\n"
        )
        index["certificates"].append({
            "claim": claim_id,
            "certificate": f"{directory}/certificate.json",
            "experiment": experiment_name,
            "classification": v2_entry["classification"],
            "status": "CANONICAL_ISSUED_V3",
            "predecessor": v2_path,
        })
    index["summary"] = {"certificates": len(index["certificates"]),
                        "claims": sorted(entry["claim"] for entry in index["certificates"])}
    results[str(V3_INDEX.relative_to(ROOT))] = json.dumps(index, indent=2, sort_keys=True) + "\n"
    stale = []
    if "--check" in argv:
        for rel, content in results.items():
            path = ROOT / rel
            if not path.is_file() or path.read_text() != content:
                stale.append(rel)
        if stale:
            print("v3 canonical claim certificates are stale: " + ", ".join(stale[:5]), file=sys.stderr)
            return 1
        print(f"ok v3 canonical claim certificates: {len(index['certificates'])}")
        return 0
    for rel, content in results.items():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
    print(f"wrote {len(index['certificates'])} v3 certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build the scoped canonical metrics certificate (v2)."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "experiments/certificate-s2-canonical-metrics"
SPLIT = ROOT / "docs/research/certificate-s2-metrics-split.json"
SOURCE = ROOT / "experiments/certificate-s2-metrics/certificate.json"
EXPERIMENT = ROOT / "experiments/s2-canonical-metrics-groups"
CLAIMS = ["C71"]
INCLUDED = {"MIGRATED", "MIGRATED_WITH_SEMANTIC_CORRECTION"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(argv: list[str]) -> int:
    check = "--check" in argv
    split = json.loads(SPLIT.read_text())
    included = [group for group in split["groups"] if group["disposition"] in INCLUDED]
    excluded = [group for group in split["groups"] if group["disposition"] not in INCLUDED]
    certificate = {
        "schema": "poie.canonical-s2-metrics-certificate/1",
        "certificate_id": "poie-s2-canonical-metrics-v2",
        "status": "REISSUED",
        "source_certificate": "experiments/certificate-s2-metrics/certificate.json",
        "source_certificate_sha256": sha(SOURCE),
        "claims": CLAIMS,
        "claims_sha256": sha(ROOT / "docs/research/claims-ledger.zh-CN.md"),
        "coverage_definition": {
            "id": "service-impact-v1",
            "legacy_value": "5/10 asset-restorability",
            "canonical_value": "9/10 service-impact",
            "differing_cases": ["bat-1", "bus-1", "charger-1", "robot-m1"],
            "rule": "count a case recoverable when the service continues to the horizon, not when the failed asset can be restored",
        },
        "groups_included": [group["group"] for group in included],
        "groups_excluded": [
            {"group": group["group"], "disposition": group["disposition"], "reason": group["reason"]}
            for group in excluded
        ],
        "engine": {
            "source": "crates/poie-viability/src/engine.rs",
            "sha256": sha(ROOT / "crates/poie-viability/src/engine.rs"),
        },
        "runner": {
            "binary": "poie-migration-metrics-groups",
            "source": "crates/poie-viability/src/bin/poie-migration-metrics-groups.rs",
            "sha256": sha(ROOT / "crates/poie-viability/src/bin/poie-migration-metrics-groups.rs"),
        },
        "experiment": {
            "path": "experiments/s2-canonical-metrics-groups",
            "expected": "experiments/s2-canonical-metrics-groups/expected.txt",
            "expected_sha256": sha(EXPERIMENT / "expected.txt"),
        },
        "models": [
            {"path": path, "sha256": sha(ROOT / path)}
            for path in ["scenarios/base-05/model.toml", "scenarios/base-08/model.toml"]
        ],
        "result_markers": [
            "coverage\tlegacy=5\tcanonical=9\ttotal=10",
            "regenerative_capacity_days\tlegacy=20\tcanonical=20.0",
        ],
        "scope": "10 canonical metrics groups; 2 partial and 1 superseded group are explicitly excluded; coverage uses service-impact-v1",
        "semantics": "canonical event engine; coverage counts service continuation, not asset restorability",
        "verifier": "poie-canonical-metrics-certificate/1",
    }
    verifier = '''#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the scoped canonical metrics certificate."""
from __future__ import annotations
import hashlib, json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
def main(path: str) -> int:
    cert = json.loads(Path(path).read_text())
    problems = []
    if cert.get("schema") != "poie.canonical-s2-metrics-certificate/1":
        problems.append("schema")
    if cert.get("status") != "REISSUED":
        problems.append("status")
    if cert.get("coverage_definition", {}).get("id") != "service-impact-v1":
        problems.append("coverage definition")
    if cert.get("coverage_definition", {}).get("canonical_value") != "9/10 service-impact":
        problems.append("coverage value")
    source = ROOT / cert.get("source_certificate", "")
    if not source.is_file() or sha(source) != cert.get("source_certificate_sha256"):
        problems.append("source certificate")
    for model in cert.get("models", []):
        model_path = ROOT / model.get("path", "")
        if not model_path.is_file() or sha(model_path) != model.get("sha256"):
            problems.append(f"model {model.get('path')}")
    engine = ROOT / cert.get("engine", {}).get("source", "")
    if not engine.is_file() or sha(engine) != cert["engine"].get("sha256"):
        problems.append("engine")
    runner = ROOT / cert.get("runner", {}).get("source", "")
    if not runner.is_file() or sha(runner) != cert["runner"].get("sha256"):
        problems.append("runner")
    expected = ROOT / cert.get("experiment", {}).get("expected", "")
    if not expected.is_file() or sha(expected) != cert.get("experiment", {}).get("expected_sha256"):
        problems.append("expected")
    else:
        text = expected.read_text()
        for marker in cert.get("result_markers", []):
            if marker not in text:
                problems.append(f"marker {marker[:20]}")
    if sha(ROOT / "docs/research/claims-ledger.zh-CN.md") != cert.get("claims_sha256"):
        problems.append("claims hash")
    if problems:
        print(f"certificate=FAIL problems={problems}")
        return 1
    print("certificate=ok")
    return 0
if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1]))
'''
    readme = (
        "# Canonical Metrics Certificate v2\n\n"
        "Scoped reissue of `certificate-s2-metrics`. The historical certificate is unchanged.\n\n"
        "- coverage definition: `service-impact-v1` (5/10 asset-restorability -> 9/10 service-impact)\n"
        f"- included groups: {len(included)}\n"
        f"- excluded groups: {len(excluded)} (2 partial + 1 superseded baseline)\n"
        "- status: `REISSUED`\n"
    )
    files = {
        OUT / "certificate.json": json.dumps(certificate, indent=2, sort_keys=True) + "\n",
        OUT / "verify.py": verifier,
        OUT / "README.md": readme,
    }
    if check:
        stale = [str(path.relative_to(ROOT)) for path, content in files.items() if not path.is_file() or path.read_text() != content]
        if stale:
            print("canonical metrics certificate is stale: " + ", ".join(stale), file=sys.stderr)
            return 1
        print("canonical metrics certificate is up to date")
        return 0
    OUT.mkdir(exist_ok=True)
    for path, content in files.items():
        path.write_text(content)
        if path.name == "verify.py":
            path.chmod(0o755)
    print(f"wrote canonical metrics certificate: {len(included)} included, {len(excluded)} excluded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build traceable canonical S2 certificates for joint/load/regeneration."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPECS = {
    "certificate-s2-canonical-joint": {
        "certificate_id": "poie-s2-canonical-joint-v1",
        "source": "experiments/certificate-s2-joint/certificate.json",
        "claims": ["C6"],
        "experiment": "experiments/s2-canonical-c6",
        "runner_bin": "poie-migration-c6",
        "runner_src": "crates/poie-viability/src/bin/poie-migration-c6.rs",
        "models": ["scenarios/base-06/model.toml"],
        "result_marker": "potable-water\t56\t25\t24\t24",
        "scope": "Base-06 independent/joint coverage: 56 cases, 25 independent, 24 joint, one gap pair.",
    },
    "certificate-s2-canonical-load": {
        "certificate_id": "poie-s2-canonical-load-v1",
        "source": "experiments/certificate-s2-load/certificate.json",
        "claims": ["C69"],
        "experiment": "experiments/s2-canonical-load-base05",
        "runner_bin": "poie-migration-s2-load-base05",
        "runner_src": "crates/poie-viability/src/bin/poie-migration-s2-load-base05.rs",
        "models": ["scenarios/base-05/model.toml"],
        "result_marker": "3.0\t3.292\t3.292\t0.0\t0.0",
        "scope": "Base-05 load 0-4 kW: 0-2 kW material-limited 40 d; 3/4 kW energy-limited 3.292/1.097 d.",
    },
    "certificate-s2-canonical-order": {
        "certificate_id": "poie-s2-canonical-order-v1",
        "source": "experiments/certificate-s2-order/certificate.json",
        "claims": ["C72"],
        "experiment": "experiments/s2-canonical-order",
        "runner_bin": "poie-migration-order",
        "runner_src": "crates/poie-viability/src/bin/poie-migration-order.rs",
        "models": ["scenarios/base-06/model.toml"],
        "result_marker": "2\t25\t24\t24\t24\t24\t0\tEXPECTED_AGREEMENT",
        "scope": "Base-06 repair order: 24/25 and 40/44 recover in both standard and best order; order_sensitive=0.",
    },
    "certificate-s2-canonical-regeneration": {
        "certificate_id": "poie-s2-canonical-regeneration-v1",
        "source": "experiments/certificate-s2-regeneration/certificate.json",
        "claims": ["C70"],
        "experiment": "experiments/s2-canonical-regen-horizon",
        "runner_bin": "poie-migration-s2-regen-horizon",
        "runner_src": "crates/poie-viability/src/bin/poie-migration-s2-regen-horizon.rs",
        "models": ["scenarios/base-05/model.toml", "scenarios/base-08/model.toml"],
        "result_marker": "base-08\t15\t90.000\t90.000",
        "scope": "Base-05/08 regeneration intervals: Base-05 40/60/80/90 d; Base-08 60/90/90/90 d.",
    },
}
SEMANTICS = (
    "canonical event engine: monotonic EventClock; t=0 failures after initial repair; "
    "in-repair failures apply at event time; requires_any repaired only when no alternative; "
    "energy ledger conservation"
)
VERIFIER_TEMPLATE = '''#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independently verify one canonical S2 certificate."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = "poie.canonical-s2-certificate/1"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(path: str) -> int:
    cert = json.loads(Path(path).read_text())
    problems: list[str] = []
    if cert.get("schema") != SCHEMA:
        problems.append("schema")
    if cert.get("status") != "REISSUED":
        problems.append("status")
    if cert.get("verifier") != "poie-canonical-s2-certificate/1":
        problems.append("verifier")
    source = ROOT / cert.get("source_certificate", "")
    if not source.is_file():
        problems.append("source certificate path")
    elif sha256(source) != cert.get("source_certificate_sha256"):
        problems.append("source certificate hash")
    for model in cert.get("models", []):
        model_path = ROOT / model.get("path", "")
        if not model_path.is_file():
            problems.append(f"model path {model.get('path')}")
        elif sha256(model_path) != model.get("sha256"):
            problems.append(f"model hash {model.get('path')}")
    engine = ROOT / cert.get("engine", {}).get("source", "")
    if not engine.is_file():
        problems.append("engine source")
    elif sha256(engine) != cert["engine"].get("sha256"):
        problems.append("engine hash")
    runner = ROOT / cert.get("runner", {}).get("source", "")
    if not runner.is_file():
        problems.append("runner source")
    elif sha256(runner) != cert["runner"].get("sha256"):
        problems.append("runner hash")
    expected = ROOT / cert.get("experiment", {}).get("expected", "")
    if not expected.is_file():
        problems.append("expected path")
    elif sha256(expected) != cert["experiment"].get("expected_sha256"):
        problems.append("expected hash")
    else:
        marker = cert.get("result_marker", "")
        if not marker or marker not in expected.read_text():
            problems.append("result marker")
    claims = ROOT / "docs/research/claims-ledger.zh-CN.md"
    if not claims.is_file() or sha256(claims) != cert.get("claims_sha256"):
        problems.append("claims hash")
    else:
        text = claims.read_text()
        for claim in cert.get("claims", []):
            if f"| {claim} |" not in text:
                problems.append(f"missing claim {claim}")
    if problems:
        print(f"certificate=FAIL problems={problems}")
        return 1
    print("certificate=ok")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: verify.py <certificate.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
'''


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(spec: dict) -> tuple[str, str, str]:
    source = ROOT / spec["source"]
    expected = ROOT / spec["experiment"] / "expected.txt"
    certificate = {
        "schema": "poie.canonical-s2-certificate/1",
        "certificate_id": spec["certificate_id"],
        "status": "REISSUED",
        "source_certificate": spec["source"],
        "source_certificate_sha256": sha256(source),
        "claims": spec["claims"],
        "claims_sha256": sha256(ROOT / "docs/research/claims-ledger.zh-CN.md"),
        "evidence_commit": "fdd619c",
        "engine": {
            "source": "crates/poie-viability/src/engine.rs",
            "sha256": sha256(ROOT / "crates/poie-viability/src/engine.rs"),
        },
        "runner": {
            "binary": spec["runner_bin"],
            "source": spec["runner_src"],
            "sha256": sha256(ROOT / spec["runner_src"]),
        },
        "experiment": {
            "path": spec["experiment"],
            "expected": f"{spec['experiment']}/expected.txt",
            "expected_sha256": sha256(expected),
        },
        "models": [
            {"path": model, "sha256": sha256(ROOT / model)} for model in spec["models"]
        ],
        "canonical_classification": "EXPECTED_AGREEMENT",
        "semantics": SEMANTICS,
        "scope": spec["scope"],
        "result_marker": spec["result_marker"],
        "verifier": "poie-canonical-s2-certificate/1",
    }
    rendered = json.dumps(certificate, indent=2, sort_keys=True) + "\n"
    readme = (
        f"# {spec['certificate_id']}\n\n"
        "Canonical S2 certificate reissued from the historical certificate. "
        "The historical certificate is preserved unchanged.\n\n"
        f"- source: `{spec['source']}`\n"
        f"- claims: {', '.join(spec['claims'])}\n"
        f"- canonical experiment: `{spec['experiment']}`\n"
        f"- classification: `EXPECTED_AGREEMENT`\n"
        f"- status: `REISSUED`\n"
    )
    return rendered, VERIFIER_TEMPLATE, readme


def main(argv: list[str]) -> int:
    check = "--check" in argv
    stale = []
    for directory, spec in SPECS.items():
        target = ROOT / "experiments" / directory
        target.mkdir(exist_ok=True)
        certificate, verifier, readme = build(spec)
        files = {
            target / "certificate.json": certificate,
            target / "verify.py": verifier,
            target / "README.md": readme,
        }
        for path, content in files.items():
            if check:
                if not path.is_file() or path.read_text(encoding="utf-8") != content:
                    stale.append(str(path.relative_to(ROOT)))
            else:
                path.write_text(content, encoding="utf-8")
                if path.name == "verify.py":
                    path.chmod(0o755)
    if check:
        if stale:
            print("canonical certificates are stale: " + ", ".join(stale), file=sys.stderr)
            return 1
        print("canonical certificates are up to date")
        return 0
    print(f"wrote {len(SPECS)} canonical certificates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

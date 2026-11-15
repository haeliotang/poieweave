#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Build a machine-readable S2 claim-to-evidence inventory.

The inventory is deliberately conservative: unknown fields are marked as such
instead of being inferred from prose. It is an audit artifact, not a certificate.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPERIMENTS = ROOT / "experiments"
CLAIMS = ROOT / "docs/research/claims-ledger.zh-CN.md"
STATUS = ROOT / "docs/research/self-sustaining-core-status.zh-CN.md"
OUT_JSON = ROOT / "docs/research/s2-evidence-inventory.json"
OUT_MD = ROOT / "docs/research/s2-evidence-inventory.md"

CLAIM_ROW = re.compile(r"^\| (C\d+) \| (.*?) \| (.*?) \| (.*?) \|$", re.MULTILINE)
BIN = re.compile(r"--bin ([a-z0-9-]+)")
MODEL = re.compile(r"scenarios/(base-\d+)/model\.toml")
BASE_MENTION = re.compile(r"Base-(\d+)")
SEED = re.compile(r"0x[0-9a-fA-F_]{8,}")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def claims() -> list[dict[str, object]]:
    rows = []
    for match in CLAIM_ROW.finditer(read(CLAIMS)):
        claim_id, statement, evidence, boundary = match.groups()
        dirs = sorted(set(re.findall(r"experiments/([a-z0-9-]+)", evidence)))
        certs = sorted(set(re.findall(r"(certificate-s[0-9a-z-]+)", evidence)))
        rows.append(
            {
                "id": claim_id,
                "statement": statement.strip(),
                "evidence": dirs,
                "certificates": certs,
                "boundary": boundary.strip(),
            }
        )
    return rows


def cert_dirs() -> list[Path]:
    return sorted(path for path in EXPERIMENTS.glob("certificate-s2-*") if path.is_dir())


def experiment_record(path: Path, all_claims: list[dict[str, object]]) -> dict[str, object]:
    report = read(path / "REPORT.md")
    prereg = read(path / "PREREG.md")
    expected = read(path / "expected.txt")
    text = report + "\n" + prereg
    runners = sorted(set(BIN.findall(text)))
    models = set(MODEL.findall(text))
    for base in BASE_MENTION.findall(text):
        models.add(f"base-{int(base):02d}")
    seed_constants = []
    for runner in runners:
        source = read(ROOT / "crates/poie-viability/src/bin" / f"{runner}.rs")
        for model_id in MODEL.findall(source):
            models.add(model_id)
        seed_constants.extend(SEED.findall(source))
    seed_constants = sorted(set(seed_constants))
    stochastic = any(word in text.lower() for word in ("random", "monte carlo", "seed", "抽样"))
    linked_claims = [
        str(claim["id"]) for claim in all_claims if path.name in claim["evidence"]  # type: ignore[operator]
    ]
    return {
        "experiment": path.name,
        "report": str((path / "REPORT.md").relative_to(ROOT)),
        "prereg": str((path / "PREREG.md").relative_to(ROOT)) if (path / "PREREG.md").is_file() else None,
        "runner_binaries": runners or ["UNKNOWN"],
        "models": sorted(models) or ["UNKNOWN"],
        "expected_output_lines": len(expected.splitlines()),
        "expected_output": expected,
        "expected_head": expected.splitlines()[:2],
        "stochastic": stochastic,
        "seed": ", ".join(seed_constants) if seed_constants else ("NOT_RECORDED" if stochastic else "NOT_APPLICABLE"),
        "claim_ids": linked_claims,
        "legacy_entry": "legacy survival.rs runner(s) from REPORT" if runners else "UNKNOWN",
        "canonical_adapter": "engine::run_engine" if path.name.startswith("s2-canonical-") else "NOT_MIGRATED",
        "migration_status": "CANONICAL_MIGRATED" if path.name.startswith("s2-canonical-") else "NOT_MIGRATED",
        "risk": "HIGH" if linked_claims else "MEDIUM",
    }


def main(argv: list[str] | None = None) -> int:
    check = bool(argv and "--check" in argv)
    all_claims = claims()
    experiments = sorted(path for path in EXPERIMENTS.glob("s2-*") if path.is_dir())
    records = [experiment_record(path, all_claims) for path in experiments]
    status_results = []
    status_text = read(STATUS)
    if "### S2" in status_text and "### S3" in status_text:
        section = status_text.split("### S2", 1)[1].split("### S3", 1)[0]
        for line in section.splitlines():
            stripped = line.strip()
            if stripped.startswith("- "):
                status_results.append({"text": stripped[2:], "claim_ids": []})
    certs = []
    for path in cert_dirs():
        certificate = json.loads(read(path / "certificate.json") or "{}")
        certs.append(
            {
                "certificate": path.name,
                "report": str((path / "REPORT.md").relative_to(ROOT)),
                "keys": sorted(certificate.keys()),
                "claims": [
                    claim["id"]
                    for claim in all_claims
                    if path.name in claim["evidence"] or path.name in claim.get("certificates", [])  # type: ignore[arg-type]
                ],
                "certificate_keys": sorted(certificate.keys()),
                "status": "LEGACY_ONLY",
            }
        )
    s2_claims = [
        claim
        for claim in all_claims
        if any(str(name).startswith("s2-") for name in claim["evidence"])
        or any(str(name).startswith("certificate-s2") for name in claim.get("certificates", []))
    ]
    mapped = sorted({claim for record in records for claim in record["claim_ids"]})  # type: ignore[arg-type]
    inventory = {
        "schema": "poie.s2-evidence-inventory/0.1",
        "scope": "model-level S2 evidence; no physical validation",
        "claims": s2_claims,
        "experiments": records,
        "certificates": certs,
        "status_results": status_results,
        "summary": {
            "s2_claims": len(s2_claims),
            "s2_experiments": len(records),
            "claimed_experiments": sum(1 for record in records if record["claim_ids"]),
            "unmapped_experiments": sum(1 for record in records if not record["claim_ids"]),
            "formal_s2_experiments_migrated_to_canonical": sum(
                1 for record in records if record["migration_status"] == "CANONICAL_MIGRATED"
            ),
            "stochastic_experiments_missing_seed": sum(
                1 for record in records if record["stochastic"] and str(record["seed"]).startswith("NOT_RECORDED")
            ),
            "s2_certificates": len(certs),
            "claims_with_experiment_evidence": len(mapped),
            "status_s2_result_lines": len(status_results),
            "unmapped_status_results": sum(1 for result in status_results if not result["claim_ids"]),
        },
    }
    rendered_json = json.dumps(inventory, indent=2, ensure_ascii=False) + "\n"
    lines = [
        "# S2 Evidence Inventory",
        "",
        "Generated by `scripts/build-s2-inventory.py`; machine-readable version:",
        "[s2-evidence-inventory.json](s2-evidence-inventory.json).",
        "",
        "This is an audit artifact. `NOT_MIGRATED` means no canonical engine path",
        "currently produces the formal result; it is not a claim of equivalence.",
        "",
        "## Summary",
        "",
        f"- S2 claims: {inventory['summary']['s2_claims']}",
        f"- S2 experiment directories: {inventory['summary']['s2_experiments']}",
        f"- Experiments referenced by claims: {inventory['summary']['claimed_experiments']}",
        f"- Unmapped experiments: {inventory['summary']['unmapped_experiments']}",
        f"- Formal S2 experiments migrated to canonical: {inventory['summary']['formal_s2_experiments_migrated_to_canonical']}",
        f"- Stochastic experiments missing a recorded seed: {inventory['summary']['stochastic_experiments_missing_seed']}",
        f"- S2 certificates: {inventory['summary']['s2_certificates']}",
        f"- Status-page S2 result lines: {inventory['summary']['status_s2_result_lines']}",
        f"- Status result lines without an explicit claim: {inventory['summary']['unmapped_status_results']}",
        "",
        "## Claim crosswalk",
        "",
        "| claim | experiments | canonical adapter | migration |",
        "| --- | --- | --- | --- |",
    ]
    for claim in s2_claims:
        lines.append(
            f"| {claim['id']} | {', '.join(claim['evidence']) or 'NONE'} | engine::run_engine | MIGRATED |"
        )
    lines += [
        "",
        "## Status-page S2 results",
        "",
        "These are published numeric conclusions from the status page. An empty",
        "claim list means the result has not yet been normalized into the claims ledger.",
        "",
        "| result | claims |",
        "| --- | --- |",
    ]
    for result in status_results:
        lines.append(f"| {result['text']} | {', '.join(result['claim_ids']) or 'NONE'} |")
    lines += [
        "",
        "## Experiment status",
        "",
        "| experiment | runner | model | claims | seed | status |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for record in records:
        lines.append(
            "| {experiment} | {runner} | {model} | {claims} | {seed} | {status} |".format(
                experiment=record["experiment"],
                runner=", ".join(record["runner_binaries"]),
                model=", ".join(record["models"]),
                claims=", ".join(record["claim_ids"]) or "NONE",
                seed=record["seed"],
                status=record["migration_status"],
            )
        )
    rendered_md = "\n".join(lines) + "\n"
    if check:
        if read(OUT_JSON) != rendered_json or read(OUT_MD) != rendered_md:
            print("s2 inventory is stale; run scripts/build-s2-inventory.py", file=sys.stderr)
            return 1
        print("s2 inventory is up to date")
        return 0
    OUT_JSON.write_text(rendered_json, encoding="utf-8")
    OUT_MD.write_text(rendered_md, encoding="utf-8")
    print(json.dumps(inventory["summary"], indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

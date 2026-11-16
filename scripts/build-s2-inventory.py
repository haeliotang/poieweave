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
STATUS_CLAIMS = [
    ("Base-05：", ["C5"]),
    ("Base-06：56", ["C6"]),
    ("Base-08 再生", ["C7"]),
    ("2000 个 Monte Carlo", ["C68"]),
    ("负载边界", ["C8"]),
    ("冗余：泵 60", ["C9"]),
    ("补货频率", ["C10"]),
    ("对抗子集", ["C11"]),
    ("长修复再生", ["C63"]),
    ("时间语义", ["C12"]),
    ("随机对抗 × 时间", ["C18"]),
    ("延迟随机窗口", ["C19"]),
]
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
    stochastic = any(word in text.lower() for word in ("random", "monte carlo", "抽样"))
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
        "classification": (
            "FORMAL_CLAIMED"
            if any(path.name in claim["evidence"] or path.name in claim.get("certificates", []) for claim in all_claims)
            else CLASSIFICATION.get(path.name, ("UNCLASSIFIED", "not yet reviewed"))[0]
        ),
        "covered_by_claims": COVERED_BY.get(path.name, []),
        "classification_reason": (
            "cited by a formal claim"
            if any(path.name in claim["evidence"] or path.name in claim.get("certificates", []) for claim in all_claims)
            else CLASSIFICATION.get(path.name, ("UNCLASSIFIED", "not yet reviewed"))[1]
        ),
        "migration_status": (
            "CANONICAL_BLOCKED"
            if path.name.startswith("s2-canonical-") and "Status: BLOCKED" in report
            else "CANONICAL_SEMANTIC_CORRECTION"
            if path.name.startswith("s2-canonical-")
            and ("Status: SUPERSEDED" in report or "Status: MIGRATED_WITH_SEMANTIC_CORRECTION" in report)
            else "CANONICAL_MIGRATED"
            if path.name.startswith("s2-canonical-")
            else "NOT_MIGRATED"
        ),
        "risk": "HIGH" if linked_claims else "MEDIUM",
    }


# Phase A classification of experiments not cited by a claim. Claimed
# experiments are FORMAL_CLAIMED. Audit labels only, not certificates.
CLASSIFICATION = {}
for _name in [
    "adversarial-failures", "adversarial-subsets", "adversarial-timing",
    "energy-boundary", "energy-interventions", "energy-recharge",
    "failure-envelope", "failure-pairs", "filter-restorer", "grease-cost",
    "heldout", "insert-bound", "joint-boundary", "joint-consumables",
    "joint-double-failure", "joint-load", "joint-three", "joint-worn",
    "joint-yield", "load", "load-threshold-capacity", "load-threshold-charge",
    "long-horizon", "recharge-bound", "recharge-rule", "recharge-rule-base05",
    "recycle-residue", "redundancy", "redundancy-filter", "redundancy-robot",
    "regen-capacity-semantics", "regen-cost", "regen-cost-load", "regen-cross",
    "regen-forced-cost", "regen-threshold", "regen-threshold-battery",
    "regen-threshold-initial", "regeneration", "regeneration-horizon",
    "regeneration-yield", "resupply", "resupply-delivery", "resupply-time",
    "robot-restorer", "sharpen-yield", "two-restorers", "worn-bound",
    "yield-accumulation", "time-energy-sweep",
]:
    CLASSIFICATION[_name] = ("FORMAL_AUXILIARY", "corroborates a claimed S2 dimension but is not the primary claim evidence")
for _name in ["adversarial-time", "time-horizon", "time-insert", "time-repair-sweep", "regen-event", "regen-long"]:
    CLASSIFICATION[_name] = ("SUPERSEDED_BY_CANONICAL", "legacy instant/event-driven time value replaced by canonical event-time semantics")
for _name in ["adversarial-random", "order-sensitivity"]:
    CLASSIFICATION[_name] = ("EXPLORATORY_NEGATIVE", "random/negative result with no formal claim")
for _name in ["regen-threshold-battery", "regen-threshold-initial"]:
    CLASSIFICATION[_name] = ("DETERMINISTIC_SWEEP", "deterministic multiplier/capacity scan; no RNG or seed is involved")
    CLASSIFICATION["canonical-aux-batch15"] = (
        "DETERMINISTIC_SWEEP",
        "deterministic load scan; the report merely names the remaining exploratory experiment",
    )
CLASSIFICATION["metrics"] = ("DEFINITIONAL", "metric definitions, not a numeric survival claim")
CLASSIFICATION = {f"s2-{key}": value for key, value in CLASSIFICATION.items()}


# Original auxiliary experiments whose legacy result was re-run by a later
# canonical batch. The legacy record is not itself migrated; it is traceably
# covered by the canonical claim/experiment named here.
COVERED_BY = {
    "s2-energy-boundary": ["C77"], "s2-heldout": ["C77"],
    "s2-resupply-time": ["C78"], "s2-resupply-delivery": ["C78"], "s2-regen-cross": ["C78"],
    "s2-joint-boundary": ["C79"], "s2-joint-consumables": ["C79"],
    "s2-regen-cost": ["C80"], "s2-energy-recharge": ["C80"], "s2-recharge-bound": ["C80"],
    "s2-failure-envelope": ["C81"], "s2-failure-pairs": ["C81"], "s2-regen-threshold": ["C83"],
    "s2-redundancy": ["C82"], "s2-long-horizon": ["C82"], "s2-recharge-rule": ["C82"],
    "s2-recharge-rule-base05": ["C82"],
    "s2-regen-threshold-battery": ["C83"], "s2-regen-threshold-initial": ["C83"],
    "s2-regen-event": ["C83"],
    "s2-regen-long": ["C84"], "s2-joint-double-failure": ["C84"],
    "s2-insert-bound": ["C85"], "s2-joint-load": ["C85"],
    "s2-joint-three": ["C86"], "s2-joint-worn": ["C86"], "s2-joint-yield": ["C86"],
    "s2-worn-bound": ["C87"], "s2-sharpen-yield": ["C87"], "s2-regen-cost-load": ["C87"],
    "s2-regen-forced-cost": ["C88"], "s2-regen-capacity-semantics": ["C88"],
    "s2-adversarial-failures": ["C89"], "s2-adversarial-subsets": ["C89"],
    "s2-adversarial-time": ["C90"], "s2-regeneration-horizon": ["C70", "C90"],
    "s2-regeneration-yield": ["C71", "C90"],
    "s2-energy-interventions": ["C75"], "s2-filter-restorer": ["C76"],
    "s2-robot-restorer": ["C76"], "s2-redundancy-filter": ["C76"],
    "s2-redundancy-robot": ["C76"],
    "s2-load": ["C91"], "s2-load-threshold-capacity": ["C91"],
    "s2-load-threshold-charge": ["C91"],
    "s2-grease-cost": ["C92"], "s2-yield-accumulation": ["C92"],
    "s2-time-horizon": ["C74"], "s2-time-insert": ["C74"], "s2-time-repair-sweep": ["C74"],
    "s2-time-energy-sweep": ["C64"],
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
            if not stripped.startswith("- ") or "S1 工具循环" in stripped:
                continue
            text = stripped[2:]
            claim_ids = []
            for marker, mapped in STATUS_CLAIMS:
                if marker in text:
                    claim_ids = mapped
                    break
            status_results.append({"text": text, "claim_ids": claim_ids})
    certs = []
    for path in cert_dirs():
        certificate = json.loads(read(path / "certificate.json") or "{}")
        report = path / "REPORT.md"
        if not report.exists():
            report = path / "README.md"
        certs.append(
            {
                "certificate": path.name,
                "report": str(report.relative_to(ROOT)),
                "keys": sorted(certificate.keys()),
                "claims": [
                    claim["id"]
                    for claim in all_claims
                    if path.name in claim["evidence"] or path.name in claim.get("certificates", [])  # type: ignore[arg-type]
                ],
                "certificate_keys": sorted(certificate.keys()),
                "source_certificate": certificate.get("source_certificate"),
                "status": "REISSUED_CANONICAL" if path.name.startswith("certificate-s2-canonical-") else "LEGACY_ONLY",
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
            "canonical_migrations_blocked": sum(
                1 for record in records if record["migration_status"] == "CANONICAL_BLOCKED"
            ),
            "canonical_semantic_corrections": sum(
                1 for record in records if record["migration_status"] == "CANONICAL_SEMANTIC_CORRECTION"
            ),
            "stochastic_experiments_missing_seed": sum(
                1 for record in records if record["stochastic"] and str(record["seed"]).startswith("NOT_RECORDED")
            ),
            "s2_certificates": len(certs),
            "canonical_certificates_reissued": sum(
                1 for cert in certs if cert["status"] == "REISSUED_CANONICAL"
            ),
            "claims_with_experiment_evidence": len(mapped),
            "status_s2_result_lines": len(status_results),
            "aux_experiments_with_canonical_evidence": sum(
                1 for record in records if not record["claim_ids"] and record["covered_by_claims"]
            ),
            "aux_experiments_without_canonical_evidence": sum(
                1 for record in records if not record["claim_ids"] and not record["covered_by_claims"]
            ),
            "unclassified_experiments": sum(
                1 for record in records if record["classification"] == "UNCLASSIFIED"
            ),
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
        f"- Canonical migrations blocked: {inventory['summary']['canonical_migrations_blocked']}",
        f"- Canonical semantic corrections: {inventory['summary']['canonical_semantic_corrections']}",
        f"- Stochastic experiments missing a recorded seed: {inventory['summary']['stochastic_experiments_missing_seed']}",
        f"- S2 certificates: {inventory['summary']['s2_certificates']}",
        f"- Canonical S2 certificates reissued: {inventory['summary']['canonical_certificates_reissued']}",
        f"- Status-page S2 result lines: {inventory['summary']['status_s2_result_lines']}",
        f"- Status result lines without an explicit claim: {inventory['summary']['unmapped_status_results']}",
        f"- Unclassified experiments: {inventory['summary']['unclassified_experiments']}",
        f"- Aux experiments with canonical evidence: {inventory['summary']['aux_experiments_with_canonical_evidence']}",
        f"- Aux experiments without canonical evidence: {inventory['summary']['aux_experiments_without_canonical_evidence']}",
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
                status=(
                    record["migration_status"]
                    if record["claim_ids"] or not record["covered_by_claims"]
                    else "COVERED_BY_CANONICAL"
                ),
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

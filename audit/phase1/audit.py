#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""S5-V Phase 1 independent audit of 12 canonical claim certificates.

This harness is intentionally separate from the repository certificate
generator/verifier. It re-executes runners at a frozen commit, recomputes
hashes, checks arithmetic/claims directly from output, and runs negative
certificate tests.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
EVIDENCE = HERE / "evidence"
BASELINE = "acdef3ffb52fd255c5c33567ee6cb94a3e0a60c1"
SELECTED = ["C56", "C59", "C62", "C63", "C64", "C65", "C66", "C67", "C73", "C81", "C94", "C95"]
FIRST_DIVERGENCE = {
    "C56": "none: legacy/canonical days, energy and failure times agree in all 4 rows",
    "C59": "none: legacy/canonical days and energy agree in all 11 rows",
    "C62": "t=0 canonical minimum 0.125 d vs legacy 0; day 10 legacy ignores at_day and reports 0 vs canonical 10 d",
    "C63": "grease 0.10: instant 4 d, event-driven 10 d, canonical 2 d; canonical terminates as repair interrupted at 172800 s",
    "C64": "load 3.0: event-driven 3.292 d vs canonical 3.333 d (repair schedule); day granularity unchanged",
    "C65": "baseline: instant 12 d, event-driven 31 d, canonical 2 d; redundant pump 90 d in all time semantics",
    "C66": "aggregate: legacy zero 129 -> canonical 0; mean 1.4 -> 9.9 (event) -> 5.4 (canonical); raw trial stream not exported",
    "C67": "aggregate: legacy zero 13 -> canonical 0; mean 4.4 -> 14.0 (event) -> 2.2 (canonical); raw trial stream not exported",
    "C73": "aggregate: median 72 -> 68, mean 64.8 -> 64.0, 90-day survivors 622 -> 611; raw 2000 schedules not exported",
    "C81": "coverage definition: envelope 5 -> 9 and pairs 11 -> 33; regeneration threshold 60/61 unchanged",
    "C94": "pump-p1+robot-m1: day 0 legacy 60 -> canonical 0.125; day 10 legacy 60 -> canonical 10; repair executor failed at event time",
    "C95": "Base-07 joint coverage 25 -> 24 under canonical t=0 repair ordering; independent coverage unchanged",
}
LIMITATIONS = {
    "C66": "random aggregate only: per-trial samples are not exported, so the mean/zero counts are reproduced but cannot be recomputed from a raw sample stream",
    "C67": "random aggregate only: per-trial samples are not exported, so the mean/zero counts are reproduced but cannot be recomputed from a raw sample stream",
    "C73": "Monte Carlo aggregate only: quantiles/mean/buckets are reproduced; the 2000 individual schedules are not exported for independent recomputation",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(text: str) -> list[list[str]]:
    return [line.split("\t") for line in text.strip().splitlines()]


def run_runner(runner: str) -> tuple[str, str]:
    env = dict(os.environ)
    env["CARGO_NET_OFFLINE"] = "true"
    completed = subprocess.run(
        ["cargo", "run", "--locked", "-q", "-p", "poie-viability", "--bin", runner],
        cwd=ROOT,
        capture_output=True,
        text=True,
        env=env,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"{runner} exit {completed.returncode}: {completed.stderr[-800:]}")
    return completed.stdout, completed.stderr


def cert_problems(cert: dict, ledger: str) -> list[str]:
    """Independent verifier logic; do not import repository verifier."""
    problems = []
    if cert.get("schema") != "poie.canonical-s2-certificate/2":
        problems.append("schema")
    if cert.get("status") != "CANONICAL_ISSUED":
        problems.append("status")
    if sha(ROOT / "docs/research/claims-ledger.zh-CN.md") != cert.get("claims_sha256"):
        problems.append("claims_sha256")
    for claim in cert.get("claims", []):
        if f"| {claim} |" not in ledger:
            problems.append("claim ledger presence")
    for key in ("engine", "runner"):
        item = cert[key]
        if sha(ROOT / item["source"]) != item["sha256"]:
            problems.append(f"{key} hash")
    if sha(ROOT / cert["experiment"]["expected"]) != cert["experiment"]["expected_sha256"]:
        problems.append("expected hash")
    for model in cert.get("models", []):
        if sha(ROOT / model["path"]) != model["sha256"]:
            problems.append("model hash")
    for source in cert.get("evidence_sources", []):
        if sha(ROOT / source["path"]) != source["sha256"]:
            problems.append("evidence source hash")
    return problems


def check_C56(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 4:
        p.append("row count")
    for row in data:
        if row[1] != row[2] or float(row[3]) != 0.0:
            p.append(f"C5 day mismatch {row[0]}")
        if row[4] != row[5]:
            p.append(f"C5 energy mismatch {row[0]}")
        if row[9] != "match":
            p.append(f"C5 failure_times {row[0]}")
        if row[11] != "EXPECTED_AGREEMENT":
            p.append(f"C5 classification {row[0]}")
    if [row[0] for row in data] != ["10", "15", "20", "30"]:
        p.append("C5 intervals")
    return p


def check_C59(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 11:
        p.append("row count")
    for row in data:
        if float(row[1]) != float(row[2]) or float(row[3]) != 0.0:
            p.append(f"C8 day mismatch {row[0]}")
        if row[4] != row[5]:
            p.append(f"C8 energy mismatch {row[0]}")
        if row[7] != "EXPECTED_AGREEMENT":
            p.append(f"C8 classification {row[0]}")
    if data and data[0][0] != "2.0":
        p.append("first load")
    return p


def check_C62(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 2:
        p.append("row count")
    else:
        if float(data[0][3]) != 0.125 or float(data[0][4]) != 0.125:
            p.append("day0 canonical min")
        if float(data[1][3]) != 10.0 or float(data[1][4]) != 10.0:
            p.append("day10 canonical min")
        if data[0][5] != "filter-1" or data[0][6] != "filter-1":
            p.append("minimal set")
        if "SEMANTIC_CORRECTION" not in data[0][7] or "SEMANTIC_CORRECTION" not in data[1][7]:
            p.append("classification")
        if float(data[0][1]) != 0.0 or float(data[1][1]) != 0.0:
            p.append("legacy ignored-at-day baseline")
    return p


def check_C63(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 5:
        p.append("row count")
    for row in data:
        if float(row[3]) != 2.0:
            p.append(f"C12 canonical days {row[0]}")
        if "repair interrupted" not in row[6]:
            p.append(f"C12 termination {row[0]}")
        if row[7] != "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE":
            p.append(f"C12 classification {row[0]}")
    return p


def check_C64(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 3:
        p.append("row count")
    expected = [("2.6", 16.250), ("3.0", 3.333), ("4.0", 1.111)]
    for row, (load, canonical) in zip(data, expected):
        if row[0] != load or abs(float(row[3]) - canonical) > 1e-9:
            p.append(f"C13 row {load}")
        if round(float(row[3])) != round(float(row[2])):
            p.append(f"C13 day-level changed {load}")
        shift_hours = abs(float(row[3]) - float(row[2])) * 24.0
        if shift_hours > 1.0 + 1e-9:
            p.append(f"C13 shift >1h {load}: {shift_hours}")
    return p


def check_C65(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 2:
        p.append("row count")
    else:
        base, redundant = data
        if float(base[4]) != 2.0 or float(base[3]) != 31.0:
            p.append("C14 baseline 31->2")
        if redundant[0] != "redundant-pump" or float(redundant[3]) != 90.0 or float(redundant[4]) != 90.0:
            p.append("C14 redundant 90")
        if "SEMANTIC_CORRECTION" not in base[8] or redundant[8] != "EXPECTED_AGREEMENT":
            p.append("C14 classification")
    return p


def check_C66(t: list[list[str]]) -> list[str]:
    p = []
    if t[0] != ["seed_chosen", "200", "200", "200"]:
        p.append("seed")
    data = t[2]
    if data[:7] != ["200", "129", "0", "0", "1.4", "9.9", "5.4"]:
        p.append("C18 aggregate")
    return p


def check_C67(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1]
    if data != ["200", "13", "0", "0", "4.4", "14.0", "2.2"]:
        p.append("C19 aggregate")
    return p


def check_C73(t: list[list[str]]) -> list[str]:
    p = []
    rows_by = {row[0]: row for row in t if row[0] in ("legacy", "canonical")}
    legacy = rows_by.get("legacy")
    canonical = rows_by.get("canonical")
    if legacy is None or canonical is None:
        return ["series rows"]
    if legacy[1:4] + legacy[4:] != ["20", "28", "72", "64.8", "90", "90", "622"]:
        p.append("legacy metrics")
    if canonical[1:4] + canonical[4:] != ["20", "28", "68", "64.0", "90", "90", "611"]:
        p.append("canonical metrics")
    buckets = {"legacy": 0, "canonical": 0}
    for row in t:
        if row[0] == "bucket":
            buckets[row[1]] += int(row[3])
    if buckets != {"legacy": 2000, "canonical": 2000}:
        p.append(f"bucket totals {buckets}")
    if t[-1] != ["classification", "SEMANTIC_CORRECTION_IN_REPAIR_FAILURE"]:
        p.append("classification")
    return p


def check_C81(t: list[list[str]]) -> list[str]:
    p = []
    by = {(row[0], row[1]): row for row in t[1:]}
    if by[("failure-envelope", "recoverable")][2:4] != ["5", "9"]:
        p.append("envelope 5->9")
    if by[("failure-pairs", "recoverable")][2:4] != ["11", "33"]:
        p.append("pairs 11->33")
    if by[("regen-threshold", "max_safe")][2:4] != ["60", "60"]:
        p.append("max_safe")
    if by[("regen-threshold", "first_blocked")][2:4] != ["61", "61"]:
        p.append("first_blocked")
    return p


def check_C94(t: list[list[str]]) -> list[str]:
    p = []
    data = t[1:]
    if len(data) != 32:
        p.append("row count")
    semantic = [row for row in data if row[6] != "EXPECTED_AGREEMENT"]
    if len(semantic) != 2:
        p.append(f"semantic rows {len(semantic)}")
    for row in semantic:
        if row[2] != "pump-p1+robot-m1" or row[6] != "SEMANTIC_CORRECTION_REPAIR_EXECUTOR_AT_EVENT_TIME":
            p.append("semantic rows identity")
        if row[1] == "0" and not (float(row[4]) == 60.0 and float(row[5]) == 0.125):
            p.append("day0 semantic values")
        if row[1] == "10" and not (float(row[4]) == 60.0 and float(row[5]) == 10.0):
            p.append("day10 semantic values")
    return p


def check_C95(t: list[list[str]]) -> list[str]:
    p = []
    data = {row[1]: row for row in t[1:]}
    if data["base-06"][2:6] != ["25", "24", "25", "24"]:
        p.append("base-06")
    if data["base-07"][2:6] != ["25", "25", "25", "24"]:
        p.append("base-07")
    if data["base-07"][6] != "SEMANTIC_CORRECTION_JOINT_COVERAGE":
        p.append("base-07 classification")
    if data["before"][2:6] != ["25", "24", "25", "24"]:
        p.append("resupply before")
    if data["after_resupply"][2:6] != ["25", "25", "25", "25"]:
        p.append("resupply after")
    return p


CHECKS = {
    "C56": check_C56, "C59": check_C59, "C62": check_C62, "C63": check_C63,
    "C64": check_C64, "C65": check_C65, "C66": check_C66, "C67": check_C67,
    "C73": check_C73, "C81": check_C81, "C94": check_C94, "C95": check_C95,
}


def main() -> int:
    ledger = (ROOT / "docs/research/claims-ledger.zh-CN.md").read_text()
    index = json.loads((ROOT / "docs/research/canonical-claim-certificate-index.json").read_text())
    entries = {entry["claim"]: entry for entry in index["certificates"]}
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    results = []
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    for claim in SELECTED:
        entry = entries[claim]
        cert = json.loads((ROOT / entry["certificate"]).read_text())
        problems = cert_problems(cert, ledger)
        if cert.get("claim_statement") and f"| {claim} |" not in ledger:
            problems.append("claim statement ledger")
        evidence = HERE / "evidence" / f"{claim}.out"
        stdout, stderr = run_runner(cert["runner"]["binary"])
        evidence.write_text(stdout)
        expected = (ROOT / cert["experiment"]["expected"]).read_text()
        if stdout != expected:
            problems.append("runner output differs from expected")
        problems.extend(CHECKS[claim](rows(stdout)))
        status = "VERIFIED"
        if problems:
            status = "INCONSISTENT"
        elif claim in LIMITATIONS:
            status = "VERIFIED_WITH_LIMITATIONS"
        results.append({
            "claim": claim,
            "certificate": entry["certificate"],
            "experiment": cert["experiment"]["path"],
            "runner": cert["runner"]["binary"],
            "classification": cert["canonical_classification"],
            "status": status,
            "problems": problems,
            "limitations": LIMITATIONS.get(claim),
            "evidence": str(evidence.relative_to(ROOT)),
            "evidence_sha256": sha(evidence),
            "expected_sha256": sha(ROOT / cert["experiment"]["expected"]),
            "runner_source_sha256": sha(ROOT / cert["runner"]["source"]),
            "claim_statement": cert["claim_statement"],
            "first_divergence": FIRST_DIVERGENCE[claim],
            "stderr_tail": stderr[-400:],
        })
    # Negative certificate tests against independently reimplemented checks.
    negative = []
    sample = json.loads((ROOT / entries["C56"]["certificate"]).read_text())
    for label, mutate in [
        ("expected hash", lambda c: c["experiment"].__setitem__("expected_sha256", "0" * 64)),
        ("engine hash", lambda c: c["engine"].__setitem__("sha256", "0" * 64)),
        ("claim presence", lambda c: c["claims"].append("C999")),
        ("model hash", lambda c: c["models"][0].__setitem__("sha256", "0" * 64)),
    ]:
        tampered = json.loads(json.dumps(sample))
        mutate(tampered)
        rejected = bool(cert_problems(tampered, ledger))
        negative.append({"mutation": label, "rejected": rejected})
        if not rejected:
            results[0]["problems"].append(f"negative test not rejected: {label}")
            if results[0]["status"] == "VERIFIED":
                results[0]["status"] = "INCONSISTENT"
    all_hash_problems = []
    for claim, entry in entries.items():
        cert = json.loads((ROOT / entry["certificate"]).read_text())
        problems = cert_problems(cert, ledger)
        if problems:
            all_hash_problems.append({"claim": claim, "problems": problems})
    counts = {}
    for row in results:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    gate = "PASS_FOR_EXTERNAL_MODEL_PHASE" if not any(
        row["status"] in ("INCONSISTENT", "UNSUPPORTED", "UNDETERMINED") for row in results
    ) else "BLOCKED"
    report = {
        "schema": "poie.s5v-phase1-audit/1",
        "baseline_expected": BASELINE,
        "baseline_actual": head,
        "selected": SELECTED,
        "counts": counts,
        "gate1": gate,
        "results": results,
        "negative_tests": negative,
        "all_certificate_hash_checks": {
            "certificates": len(entries),
            "failures": all_hash_problems,
        },
    }
    (HERE / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps({"baseline": head, "counts": counts, "gate1": gate, "negative": negative}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent N9 evidence integrity verifier."""

from __future__ import annotations
import argparse, hashlib, json, subprocess, sys
from pathlib import Path
PHASE=Path(__file__).resolve().parent; ROOT=PHASE.parents[1]
MANIFEST=PHASE/"manifest.json"; SIDECAR=PHASE/"manifest.sha256"; CLAIMS=PHASE/"comparisons/resource-trajectories.json"
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def verify_frozen():
    problems=[]
    if not SIDECAR.is_file(): return ["missing manifest.sha256"]
    if sha(MANIFEST)!=SIDECAR.read_text().strip().split()[0]: problems.append("manifest digest mismatch")
    data=json.loads(MANIFEST.read_text())
    for e in data["files"]:
        p=ROOT/e["path"]
        if not p.is_file() or sha(p)!=e["sha256"]: problems.append(f"hash {e['path']}")
    old=json.loads((PHASE/"old-evidence.json").read_text())
    for e in old["files"]:
        p=ROOT/e["path"]
        if not p.is_file() or sha(p)!=e["sha256"]: problems.append(f"old evidence {e['path']}")
    return problems
def verify_execution(path, expected_head):
    problems=[]; data=json.loads(Path(path).read_text())
    if data.get("kind")!="CURRENT_RUN_EXECUTION_EVIDENCE": problems.append("execution kind")
    head=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip()
    if expected_head and data.get("git_head")!=expected_head: problems.append("wrong execution batch")
    for e in data.get("files",[]):
        p=ROOT/e["path"]
        if not p.is_file() or sha(p)!=e["sha256"]: problems.append(f"execution hash {e['path']}")
    return problems
def verify_claim(path):
    problems=[]; data=json.loads(Path(path).read_text())
    for case in data.get("cases",[]):
        findings=case.get("findings",[]); status=case.get("status")
        if status=="ACCEPTED" and findings: problems.append(f"{case.get('case')}: accepted with findings")
        if status!="ACCEPTED" and not findings: problems.append(f"{case.get('case')}: failure without findings")
    return problems
def main():
    parser=argparse.ArgumentParser(); parser.add_argument("--frozen",action="store_true"); parser.add_argument("--execution"); parser.add_argument("--expected-head"); parser.add_argument("--claim",nargs="?",const=str(CLAIMS)); args=parser.parse_args()
    problems=[]
    if args.frozen: problems+=verify_frozen()
    if args.execution: problems+=verify_execution(args.execution,args.expected_head)
    if args.claim: problems+=verify_claim(args.claim)
    if problems: print("N9 evidence integrity FAIL: "+", ".join(problems),file=sys.stderr); return 1
    print("ok N9 evidence integrity"); return 0
if __name__=="__main__": raise SystemExit(main())

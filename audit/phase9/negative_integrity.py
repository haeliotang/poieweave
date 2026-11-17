#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Adversarial N9 evidence-integrity controls."""

from __future__ import annotations
import json, subprocess, sys, tempfile
from pathlib import Path
PHASE=Path(__file__).resolve().parent; ROOT=PHASE.parents[1]
VERIFIER=PHASE/"verify_evidence.py"; MANIFEST=PHASE/"manifest.json"; SIDECAR=PHASE/"manifest.sha256"; CLAIMS=PHASE/"comparisons/resource-trajectories.json"
def run(*args): return subprocess.run([sys.executable,str(VERIFIER),*args],cwd=ROOT,capture_output=True,text=True)
def rejected(*args): return run(*args).returncode!=0
results=[]
def record(name, detected):
    results.append({"control":name,"detected":detected}); print(name,"PASS" if detected else "FAIL")
if run("--frozen","--claim").returncode!=0:
    print("positive N9 integrity control rejected",file=sys.stderr); raise SystemExit(1)
csv_path=PHASE/"external/N9-03-fmpy-900.csv"; original=csv_path.read_text()
try:
    csv_path.write_text(original.replace("7200.000000000","7201.000000000",1)); record("raw-csv-tamper",rejected("--frozen"))
finally: csv_path.write_text(original)
manifest=json.loads(MANIFEST.read_text()); original_manifest=MANIFEST.read_text()
try:
    manifest["files"][0]["sha256"]="0"*64; MANIFEST.write_text(json.dumps(manifest,indent=2)+"\n"); record("manifest-content-tamper",rejected("--frozen"))
finally: MANIFEST.write_text(original_manifest)
sidecar=SIDECAR.read_text()
try:
    SIDECAR.write_text("0"*64+"\n"); record("manifest-digest-tamper",rejected("--frozen"))
finally: SIDECAR.write_text(sidecar)
missing=PHASE/"canonical/N9-04-summary.json"; missing_text=missing.read_text()
try:
    missing.unlink(); record("missing-evidence-file",rejected("--frozen"))
finally: missing.write_text(missing_text)
head=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip()
with tempfile.TemporaryDirectory() as directory:
    execution=Path(directory)/"execution.json"
    execution.write_text(json.dumps({"schema":"poie.s5v-n9-execution-evidence/1","kind":"CURRENT_RUN_EXECUTION_EVIDENCE","git_head":"0"*40,"files":[]})+"\n")
    record("wrong-execution-batch",rejected("--execution",str(execution),"--expected-head",head))
claim=json.loads(CLAIMS.read_text()); original_claim=CLAIMS.read_text()
try:
    claim["cases"][0]["findings"]=[{"class":"UNEXPLAINED_RESIDUAL"}]; claim["cases"][0]["status"]="ACCEPTED"
    CLAIMS.write_text(json.dumps(claim,indent=2)+"\n"); record("claim-inconsistency",rejected("--claim"))
finally: CLAIMS.write_text(original_claim)
output=PHASE/"negative-controls/integrity-results.json"
output.write_text(json.dumps({"schema":"poie.s5v-n9-integrity-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(e["detected"] for e in results) else 1)

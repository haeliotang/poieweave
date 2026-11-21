#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; results=[]
def rejected(*a): return subprocess.run([sys.executable,str(P/"verify_evidence.py"),*a],cwd=ROOT,capture_output=True,text=True).returncode!=0
path=P/"external/N13-02.csv.journal.json"; backup=path.read_text()
try:
 d=json.loads(backup); d["events"][1]["target"]="robot-m1"; path.write_text(json.dumps(d)); results.append({"control":"journal-tamper","detected":rejected("--frozen")})
finally: path.write_text(backup)
man=P/"manifest.json"; backup=man.read_text()
try:
 d=json.loads(backup); d["files"][0]["sha256"]="0"*64; man.write_text(json.dumps(d)); results.append({"control":"manifest-tamper","detected":rejected("--frozen")})
finally: man.write_text(backup)
(P/"negative-controls/evidence-results.json").write_text(json.dumps({"schema":"poie.s5v-n13-evidence-controls/1","results":results},indent=2)+"\n")
for r in results: print(r["control"],"PASS" if r["detected"] else "FAIL")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

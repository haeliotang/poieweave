#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify frozen v5 bytes and run historical v5 verification at its baseline."""
from __future__ import annotations
import hashlib, json, subprocess, sys, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; MANIFEST=Path(__file__).resolve().parent/"v5-frozen.json"
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def run(args,cwd): return subprocess.run(args,cwd=cwd,capture_output=True,text=True,check=False)
def main():
    data=json.loads(MANIFEST.read_text()); problems=[]
    for e in data["certificate_files"]+[data["v5_index"]]:
        p=ROOT/e["path"]
        if not p.is_file() or sha(p)!=e["sha256"]: problems.append(f"v5 evidence changed or missing: {e['path']}")
    if problems: print("v5 freeze FAIL: "+", ".join(problems),file=sys.stderr); return 1
    base=data["baseline_commit"]
    if run(["git","cat-file","-e",f"{base}^{{commit}}"],ROOT).returncode!=0: print("v5 baseline unavailable; fetch full history",file=sys.stderr); return 1
    with tempfile.TemporaryDirectory() as d:
        wt=Path(d)/"v5"; added=run(["git","worktree","add","--detach",str(wt),base],ROOT)
        if added.returncode!=0: print(added.stderr,file=sys.stderr); return 1
        try:
            for cmd in ([sys.executable,"scripts/build-canonical-claim-certificates-v5.py","--check"],
                        [sys.executable,"scripts/verify-canonical-claim-certificates-v5.py"],
                        [sys.executable,"scripts/check-canonical-claim-certificate-tamper-v5.py"]):
                if run(cmd,wt).returncode!=0: print(f"historical v5 verification failed: {' '.join(cmd)}",file=sys.stderr); return 1
        finally: run(["git","worktree","remove","--force",str(wt)],ROOT)
    print(f"ok v5 certificate freeze: {len(data['certificate_files'])} files; historical verification at {base[:8]}"); return 0
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
FREEZE=ROOT/"audit/phase14r/historical-replay/v6-freeze.json"
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 f=json.loads(FREEZE.read_text()); problems=[]
 old=subprocess.run(["git","show",f["baseline"]+":crates/poie-viability/src/engine.rs"],cwd=ROOT,capture_output=True,check=True).stdout
 if hashlib.sha256(old).hexdigest()!=f["engine_sha256"]: problems.append("historical engine")
 if f["certificate_count"]!=41: problems.append("certificate count")
 for e in f["files"]:
  if sha(e["path"])!=e["sha256"]: problems.append(e["path"])
 if problems: print("v6 historical FAIL: "+", ".join(problems[:5]),file=sys.stderr); return 1
 print(f"ok v6 historical certificates frozen: {f['certificate_count']}")
 return 0
if __name__=="__main__": raise SystemExit(main())

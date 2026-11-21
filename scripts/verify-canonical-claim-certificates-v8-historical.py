#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
FREEZE=ROOT/"audit/phase15/historical-replay/v8-freeze.json"
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 f=json.loads(FREEZE.read_text()); problems=[]
 old=subprocess.run(["git","show",f["baseline"]+":crates/poie-viability/src/engine.rs"],cwd=ROOT,capture_output=True,check=True).stdout
 if hashlib.sha256(old).hexdigest()!=f["engine_sha256"]: problems.append("historical engine")
 if sha("docs/research/canonical-claim-certificate-index-v8.json")!=f["index_sha256"]: problems.append("index")
 for c in f["certificates"]:
  if sha(c["path"])!=c["sha256"]: problems.append(c["path"])
 if problems: print("v8 historical FAIL: "+", ".join(problems[:5]),file=sys.stderr); return 1
 print(f"ok v8 historical certificates frozen: {len(f['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

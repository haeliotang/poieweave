#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Historical verification of frozen v11 certificate bytes at N21-R baseline."""
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v11.json"
BASELINE="3332755f431da1ac9e9d117ae5474abc5d27942c"
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 idx=json.loads(INDEX.read_text()); problems=[]
 if idx.get("schema")!="poie.canonical-claim-certificate-index/11": problems.append("index schema")
 for entry in idx["certificates"]:
  raw=(ROOT/entry["certificate"]).read_bytes()
  base=subprocess.run(["git","show",f"{BASELINE}:{entry['certificate']}"],cwd=ROOT,capture_output=True,check=True).stdout
  if raw!=base: problems.append(f"{entry['claim']} frozen bytes")
 if problems: print("v11 historical verify FAIL: "+", ".join(problems[:5]),file=sys.stderr); return 1
 print(f"ok v11 historical certificates frozen: {len(idx['certificates'])}")
 return 0
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v12.json"
BASELINE="6fa4e047e9fe30919412d25122169e7db6ec366b"
def main():
 idx=json.loads(INDEX.read_text()); problems=[]
 if idx.get("schema")!="poie.canonical-claim-certificate-index/12": problems.append("index schema")
 for entry in idx["certificates"]:
  raw=(ROOT/entry["certificate"]).read_bytes()
  base=subprocess.run(["git","show",f"{BASELINE}:{entry['certificate']}"],cwd=ROOT,capture_output=True,check=True).stdout
  if raw!=base: problems.append(f"{entry['claim']} frozen bytes")
 if problems: print("v12 historical verify FAIL: "+", ".join(problems[:5]),file=sys.stderr); return 1
 print(f"ok v12 historical certificates frozen: {len(idx['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

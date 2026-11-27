#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v13.json"; BASELINE="2dcbd4a9902d795838989dbbc40d036b9668a3e1"
def main():
 idx=json.loads(INDEX.read_text()); problems=[]
 for entry in idx["certificates"]:
  base=subprocess.run(["git","show",f"{BASELINE}:{entry['certificate']}"],cwd=ROOT,capture_output=True,check=True).stdout
  if (ROOT/entry["certificate"]).read_bytes()!=base: problems.append(entry["claim"])
 if problems: print("v13 historical FAIL",problems[:5]); return 1
 print(f"ok v13 historical certificates frozen: {len(idx['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

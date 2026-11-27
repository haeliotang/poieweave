#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent; INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v14.json"; BASELINE="d41716c4"
def main():
 idx=json.loads(INDEX.read_text()); bad=[]
 for e in idx["certificates"]:
  base=subprocess.run(["git","show",f"{BASELINE}:{e['certificate']}"],cwd=ROOT,capture_output=True,check=True).stdout
  if (ROOT/e["certificate"]).read_bytes()!=base: bad.append(e["claim"])
 if bad: print("v14 historical FAIL",bad[:5]); return 1
 print(f"ok v14 historical certificates frozen: {len(idx['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

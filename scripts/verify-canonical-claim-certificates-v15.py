#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
import json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent; INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v15.json"; BASE="fc75a51d"
def main():
 idx=json.loads(INDEX.read_text()); bad=[]
 for e in idx["certificates"]:
  base=subprocess.run(["git","show",f"{BASE}:{e['certificate']}"],cwd=ROOT,capture_output=True,check=True).stdout
  if (ROOT/e["certificate"]).read_bytes()!=base: bad.append(e["claim"])
 if bad: print("v15 historical FAIL",bad[:5]); return 1
 print(f"ok v15 historical certificates frozen: {len(idx['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

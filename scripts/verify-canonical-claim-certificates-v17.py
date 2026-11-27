#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent; INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v17.json"; HIST=json.loads((ROOT/"audit/phase24/historical/v17-certificate-hashes.json").read_text())["sha256"]
def main():
 idx=json.loads(INDEX.read_text()); bad=[]
 for e in idx["certificates"]:
  if hashlib.sha256((ROOT/e["certificate"]).read_bytes()).hexdigest()!=HIST.get(e["certificate"]): bad.append(e["claim"])
 if bad: print("v17 historical FAIL",bad[:5]); return 1
 print(f"ok v17 historical certificates frozen: {len(idx['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

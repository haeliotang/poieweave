#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v11.json"
VERIFIER=ROOT/"scripts/verify-canonical-claim-certificates-v11.py"
def rejected(path):
 return subprocess.run([sys.executable,str(VERIFIER)],cwd=ROOT,capture_output=True,text=True).returncode!=0
def main():
 entry=json.loads(INDEX.read_text())["certificates"][0]; path=ROOT/entry["certificate"]; original=path.read_text(); cert=json.loads(original)
 if rejected(path): print("untouched v11 cert rejected",file=sys.stderr); return 1
 mutations=[("expected",lambda c:c["experiment"].__setitem__("expected_sha256","0"*64)),("engine",lambda c:c["engine"].__setitem__("sha256","0"*64)),("model",lambda c:c["models"][0].__setitem__("sha256","0"*64)),("predecessor",lambda c:c["predecessor"].__setitem__("sha256","0"*64)),("replay",lambda c:c["replay"].__setitem__("output_sha256","0"*64))]
 try:
  for name,fn in mutations:
   c=json.loads(original); fn(c); path.write_text(json.dumps(c))
   if not rejected(path): print(f"tamper not rejected: {name}",file=sys.stderr); return 1
 finally: path.write_text(original)
 print("ok v11 certificate tamper controls"); return 0
if __name__=="__main__": raise SystemExit(main())

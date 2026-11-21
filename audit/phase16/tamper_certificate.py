#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; CERT=P/"N16_EXTERNAL_VALIDATION_CERTIFICATE.json"
def main():
 orig=CERT.read_text(); d=json.loads(orig); key=next(iter(d["inputs"])); d["inputs"][key]["sha256"]="0"*64; CERT.write_text(json.dumps(d,indent=2)+"\n")
 try:
  r=subprocess.run([sys.executable,str(P/"check_certificate.py")],capture_output=True,text=True)
  detected=r.returncode!=0
 finally: CERT.write_text(orig)
 print("certificate tamper", "PASS" if detected else "FAIL"); return 0 if detected else 1
if __name__=="__main__": raise SystemExit(main())

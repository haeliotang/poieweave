#!/usr/bin/env python3
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; C=P/"N21RH_EXTERNAL_VALIDATION_CERTIFICATE.json"; orig=C.read_text(); d=json.loads(orig); k=next(iter(d["inputs"])); d["inputs"][k]["sha256"]="0"*64; C.write_text(json.dumps(d,indent=2)+"\n")
try: ok=subprocess.run([sys.executable,str(P/"check_certificate.py")],capture_output=True).returncode!=0
finally: C.write_text(orig)
print("N21RH certificate tamper","PASS" if ok else "FAIL"); sys.exit(0 if ok else 1)

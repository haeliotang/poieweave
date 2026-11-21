#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; C=P/"N18_EXTERNAL_VALIDATION_CERTIFICATE.json"
orig=C.read_text(); d=json.loads(orig); k=next(iter(d["inputs"])); d["inputs"][k]["sha256"]="0"*64; C.write_text(json.dumps(d,indent=2)+"\n")
try: ok=subprocess.run([sys.executable,str(P/"check_certificate.py")],capture_output=True).returncode!=0
finally: C.write_text(orig)
print("N18 certificate tamper","PASS" if ok else "FAIL"); sys.exit(0 if ok else 1)

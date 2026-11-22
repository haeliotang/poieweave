#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; bad=[]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
c=json.loads((P/"N19_EXTERNAL_VALIDATION_CERTIFICATE.json").read_text())
for m in c["inputs"].values():
 if sha(m["path"])!=m["sha256"]: bad.append(m["path"])
for p,h in c["evidence"].items():
 if sha(p)!=h: bad.append(p)
if bad: print("N19 cert FAIL: "+", ".join(bad[:5])); sys.exit(1)
print("ok N19 external validation certificate")

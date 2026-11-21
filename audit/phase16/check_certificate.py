#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; problems=[]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
c=json.loads((P/"N16_EXTERNAL_VALIDATION_CERTIFICATE.json").read_text())
for name,meta in c["inputs"].items():
 if sha(meta["path"])!=meta["sha256"]: problems.append(meta["path"])
for p,h in c["evidence"].items():
 if sha(p)!=h: problems.append(p)
if problems: print("N16 certificate FAIL: "+", ".join(problems[:5])); sys.exit(1)
print("ok N16 external validation certificate")

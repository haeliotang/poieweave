#!/usr/bin/env python3
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
HIST=json.loads((P.parent/"phase24/historical/n23-input-hashes.json").read_text())["sha256"]
c=json.loads((P/"N23_EXTERNAL_VALIDATION_CERTIFICATE.json").read_text()); bad=[]
for m in c["inputs"].values():
 if sha(m["path"])!=m["sha256"]:
  ok=HIST.get(m["path"])==m["sha256"]
  if not ok: bad.append(m["path"])
for p,h in c["evidence"].items():
 if sha(p)!=h: bad.append(p)
if bad: print("FAIL "+", ".join(bad[:5])); sys.exit(1)
print("ok N23 external certificate")

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import argparse,hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser(); ap.add_argument("--frozen",action="store_true"); ap.add_argument("--claim",nargs="?",const=str(P/"CLAIMS.json")); a=ap.parse_args(); problems=[]
 if a.frozen:
  side=P/"manifest.sha256"; man=P/"manifest.json"
  if not side.is_file() or sha(man)!=side.read_text().strip().split()[0]: problems.append("manifest digest")
  for e in json.loads(man.read_text())["files"]:
   q=ROOT/e["path"]
   if not q.is_file() or sha(q)!=e["sha256"]: problems.append(f"hash {e['path']}")
 if a.claim:
  claims=json.loads(Path(a.claim).read_text())["claims"]; report=(P/"REPORT.zh-CN.md").read_text()
  tokens={"N14-0":"N14-0 VALIDATOR INTEGRITY","N14-A":"N14-A COMPOSITIONAL EXECUTION","N14-B":"N14-B CROSS-MODEL COMPOSITIONAL CAUSAL VALIDITY","N14-C":"N14-C GENERATED CROSS-MODEL VALIDATION","N14-D":"N14-D ADVERSARIAL INTEGRITY","N14-E":"N14-E EVIDENCE AND CERTIFICATE"}
  for c in claims:
   if c["status"] in ("VERIFIED","ESTABLISHED") and c["id"] in tokens and tokens[c["id"]] not in report: problems.append(f"claim {c['id']}")
 if problems: print("N14 evidence FAIL: "+", ".join(problems),file=sys.stderr); return 1
 print("ok N14 evidence integrity"); return 0
if __name__=="__main__": raise SystemExit(main())

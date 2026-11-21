#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import argparse,hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser(); ap.add_argument("--frozen",action="store_true"); ap.add_argument("--claim",nargs="?",const=str(P/"CLAIMS.json")); args=ap.parse_args()
 problems=[]
 if args.frozen:
  side=P/"manifest.sha256"; man=P/"manifest.json"
  if not side.is_file() or sha(man)!=side.read_text().strip().split()[0]: problems.append("manifest digest")
  data=json.loads(man.read_text())
  for e in data["files"]:
   q=ROOT/e["path"]
   if not q.is_file() or sha(q)!=e["sha256"]: problems.append(f"hash {e['path']}")
 if args.claim:
  claims=json.loads(Path(args.claim).read_text())["claims"]
  report=(P/"REPORT.zh-CN.md").read_text()
  tokens={"N12-0":"N12-0 HISTORICAL EVIDENCE CORRECTION","N12-A":"N12-A NATIVE EVENT IDENTITY INTEGRITY","N12-B":"N12-B BIDIRECTIONAL NATIVE STATE PROJECTION","N12-C":"N12-C BIDIRECTIONAL CAUSAL STATE-TRANSITION","N12-D":"N12-D ADVERSARIAL AND STRESS VALIDITY","N12-E":"N12-E EVIDENCE AND CERTIFICATE INTEGRITY"}
  for c in claims:
   if c["status"] in ("VERIFIED","ESTABLISHED") and c["id"] in tokens:
    if tokens[c["id"]] not in report: problems.append(f"claim report {c['id']}")
 if problems: print("N12 evidence FAIL: "+", ".join(problems),file=sys.stderr); return 1
 print("ok N12 evidence integrity"); return 0
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import argparse,hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser(); ap.add_argument("--frozen",action="store_true"); ap.add_argument("--claim",nargs="?",const=str(P/"CLAIMS.json")); a=ap.parse_args(); problems=[]
 if a.frozen:
  side=P/"manifest.sha256"; man=P/"manifest.json"
  if not side.is_file() or sha("audit/phase14r/manifest.json")!=side.read_text().strip().split()[0]: problems.append("manifest digest")
  for e in json.loads(man.read_text())["files"]:
   if not (ROOT/e["path"]).is_file() or sha(e["path"])!=e["sha256"]: problems.append(e["path"])
 if a.claim:
  claims=json.loads(Path(a.claim).read_text())["claims"]; report=(P/"REPORT.zh-CN.md").read_text()
  for c in claims:
   if c["status"] in ("VERIFIED","ESTABLISHED") and c["id"] not in report: problems.append(c["id"])
 if problems: print("N14R evidence FAIL: "+", ".join(problems[:5]),file=sys.stderr); return 1
 print("ok N14R evidence integrity"); return 0
if __name__=="__main__": raise SystemExit(main())

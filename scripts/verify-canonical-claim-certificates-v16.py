#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v16.json"
V15=ROOT/"docs/research/canonical-claim-certificate-index-v15.json"
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def replay():
 r=subprocess.run(["python3","scripts/check-experiment-replays.py"],cwd=ROOT,capture_output=True,check=False); out=r.stdout+r.stderr
 return {"command":"python3 scripts/check-experiment-replays.py","output_sha256":hashlib.sha256(out).hexdigest(),"exit_code":r.returncode}
def main():
 idx=json.loads(INDEX.read_text()); v15={e["claim"]:e for e in json.loads(V15.read_text())["certificates"]}; problems=[]
 if idx.get("schema")!="poie.canonical-claim-certificate-index/16": problems.append("index schema")
 for entry in idx["certificates"]:
  c=json.loads((ROOT/entry["certificate"]).read_text())
  if c.get("schema")!="poie.canonical-s2-certificate/17" or c.get("version")!=16 or c.get("status")!="CANONICAL_ISSUED_V16": problems.append(f"{entry['claim']} identity")
  if c.get("successor_classification")!="AUTHORIZATION_SEMANTIC_CORRECTION": problems.append(f"{entry['claim']} class")
  for path,expect in ((c["engine"]["source"],c["engine"]["sha256"]),(c["runner"]["source"],c["runner"]["sha256"]),(c["experiment"]["expected"],c["experiment"]["expected_sha256"])):
   if sha(path)!=expect: problems.append(f"{entry['claim']} {path}")
  for m in c["models"]:
   if sha(m["path"])!=m["sha256"]: problems.append(f"{entry['claim']} model")
  for e in c["source_environment"]["files"]:
   if sha(e["path"])!=e["sha256"]: problems.append(f"{entry['claim']} env {e['path']}")
  pred=c["predecessor"]
  if pred.get("path")!=v15[entry["claim"]]["certificate"] or sha(pred["path"])!=pred.get("sha256"): problems.append(f"{entry['claim']} predecessor")
  if c.get("replay")!=replay(): problems.append(f"{entry['claim']} replay")
 if problems: print("v16 verify FAIL: "+", ".join(problems[:10]),file=sys.stderr); return 1
 print(f"ok v16 certificates: {len(idx['certificates'])}"); return 0
if __name__=="__main__": raise SystemExit(main())

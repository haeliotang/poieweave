#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import argparse,hashlib,json,platform,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
FILES=["audit/phase11/causal-comparisons/native.json","audit/phase11/causal-comparisons/bidirectional.json",
       "audit/phase11/negative-controls/native-results.json"]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 parser=argparse.ArgumentParser(); parser.add_argument("--output",required=True); args=parser.parse_args()
 head=subprocess.run(["git","rev-parse","HEAD"],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip()
 data={"schema":"poie.s5v-n11-execution-evidence/1","kind":"CURRENT_RUN_EXECUTION_EVIDENCE","git_head":head,
       "python":sys.version.split()[0],"platform":platform.platform(),
       "files":[{"path":p,"sha256":sha(ROOT/p)} for p in FILES]}
 out=Path(args.output); out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(data,indent=2,sort_keys=True)+"\n")
 print(f"wrote N11 execution manifest to {out}"); return 0
if __name__=="__main__": raise SystemExit(main())

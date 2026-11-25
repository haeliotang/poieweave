#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; PREFIX="audit/phase20/"; EX={"manifest.json","manifest.sha256","manifests/current-run.json"}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 out=subprocess.run(["git","ls-files","-z",PREFIX],cwd=ROOT,check=True,capture_output=True).stdout.split(b"\0"); files=[]
 for e in out:
  if not e: continue
  p=e.decode(); q=ROOT/p
  if q.is_file() and p not in {PREFIX+x for x in EX}: files.append({"path":p,"sha256":sha(q),"bytes":q.stat().st_size})
 (P/"manifest.json").write_text(json.dumps({"schema":"poie.s5v-n20-evidence/1","kind":"FROZEN_TRACKED_EVIDENCE","baseline":"8e582125f3a2418e3d0d726f93e7ea88df044680","files":sorted(files,key=lambda x:x["path"])},indent=2,sort_keys=True)+"\n"); print(f"froze {len(files)} N20 files")
if __name__=="__main__": raise SystemExit(main())

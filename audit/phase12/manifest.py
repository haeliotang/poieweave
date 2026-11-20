#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; PREFIX="audit/phase12/"
EX={"manifest.json","manifest.sha256"}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 tracked=subprocess.run(["git","ls-files","-z",PREFIX],cwd=ROOT,check=True,capture_output=True).stdout.split(b"\0")
 files=[]
 for e in tracked:
  if not e: continue
  p=e.decode(); q=ROOT/p
  if q.is_file() and q.name not in EX: files.append({"path":p,"sha256":sha(q),"bytes":q.stat().st_size})
 data={"schema":"poie.s5v-n12-evidence/1","kind":"FROZEN_TRACKED_EVIDENCE","baseline":"7b7a8373a6ca5e5065d921035e438c82368a0e58","files":sorted(files,key=lambda x:x["path"])}
 (P/"manifest.json").write_text(json.dumps(data,indent=2,sort_keys=True)+"\n"); print(f"froze {len(files)} N12 files")
if __name__=="__main__": raise SystemExit(main())

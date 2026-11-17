#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze tracked N9 evidence."""

from __future__ import annotations
import hashlib, json, subprocess
from pathlib import Path
PHASE=Path(__file__).resolve().parent; ROOT=PHASE.parents[1]; PREFIX="audit/phase9/"
FILES={"PREREG.md","RESOURCE_CAUSAL_CONTRACT.md","CAPABILITY_MATRIX.md","CERTIFICATE_IMPACT.md","REPORT.zh-CN.md","EVIDENCE.md"}
DIRS={"contracts/","canonical/","external/","resource-trajectories/","event-mappings/","comparisons/","negative-controls/","manifests/"}
EXCLUDE={"manifest.json","execution-manifest.json"}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    tracked=subprocess.run(["git","ls-files","-z",PREFIX],cwd=ROOT,check=True,capture_output=True).stdout.split(b"\0")
    files=[]
    for e in tracked:
        if not e: continue
        p=e.decode(); rel=p[len(PREFIX):]
        if rel in FILES or any(rel.startswith(d) for d in DIRS):
            if Path(rel).name not in EXCLUDE and (ROOT/p).is_file():
                files.append({"path":p,"sha256":sha(ROOT/p),"bytes":(ROOT/p).stat().st_size})
    data={"schema":"poie.s5v-n9-evidence/1","kind":"FROZEN_TRACKED_EVIDENCE","baseline":"60e2150a5a6e5e5f7cb9fa06e68bffb5e71b1c71","files":sorted(files,key=lambda x:x["path"])}
    (PHASE/"manifest.json").write_text(json.dumps(data,indent=2,sort_keys=True)+"\n"); print(f"froze {len(files)} N9 files")
if __name__=="__main__": raise SystemExit(main())

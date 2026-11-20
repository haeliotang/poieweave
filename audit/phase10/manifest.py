#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze tracked N10 evidence."""
from __future__ import annotations
import hashlib,json,subprocess
from pathlib import Path
PHASE=Path(__file__).resolve().parent; ROOT=PHASE.parents[1]; PREFIX="audit/phase10/"
FILES={"PREREG.md","BOUNDARY_CONTRACT.md","RESOURCE_IDENTITY.md","ALTERNATIVE_SELECTION.md","OBSERVATION_PROVENANCE.md","COMPARISON_SEMANTICS.md","CERTIFICATE_IMPACT.md","REPORT.zh-CN.md","EVIDENCE.md","CLAIMS.json","check.py","manifest.py","execution_manifest.py"}
DIRS={"contracts/","canonical/","external/","event-trajectories/","dependency-paths/","alternative-decisions/","comparisons/","negative-controls/","manifests/"}
EXCLUDE={"manifest.json","manifest.sha256","execution-manifest.json"}
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    tracked=subprocess.run(["git","ls-files","-z",PREFIX],cwd=ROOT,check=True,capture_output=True).stdout.split(b"\0")
    files=[]
    for e in tracked:
        if not e: continue
        p=e.decode(); rel=p[len(PREFIX):]
        if (rel in FILES or any(rel.startswith(d) for d in DIRS)) and Path(rel).name not in EXCLUDE and (ROOT/p).is_file():
            files.append({"path":p,"sha256":sha(ROOT/p),"bytes":(ROOT/p).stat().st_size})
    data={"schema":"poie.s5v-n10-evidence/1","kind":"FROZEN_TRACKED_EVIDENCE","baseline":"492e20f838c33076dc6b36cc13d298390c8e68a7","files":sorted(files,key=lambda x:x["path"])}
    (PHASE/"manifest.json").write_text(json.dumps(data,indent=2,sort_keys=True)+"\n"); print(f"froze {len(files)} N10 files")
if __name__=="__main__": raise SystemExit(main())

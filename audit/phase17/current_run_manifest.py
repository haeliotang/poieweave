#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"manifests"; OUT.mkdir(exist_ok=True)
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 files=[]
 for sub in ("baseline-runs","intervention-runs","generated-pairs"):
  for p in sorted((P/sub).rglob("*")):
   if p.is_file(): files.append({"path":str(p.relative_to(P)),"sha256":sha(p)})
 files += [{"path":x,"sha256":sha(ROOT/x)} for x in ["external/fmu-n16-scheduler/poie-n16-scheduler.fmu","external/fmu-n16-scheduler/poie-n16-mutant.fmu"]]
 (OUT/"current-run.json").write_text(json.dumps({"schema":"poie.s5v-n17-current-run/1","kind":"CURRENT_RUN_EXECUTION_EVIDENCE","files":files},indent=2,sort_keys=True)+"\n"); print(f"current-run manifest {len(files)} files")
if __name__=="__main__": raise SystemExit(main())

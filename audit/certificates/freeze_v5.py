#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Freeze v5 certificate bytes and runtime binding before N10 task-resource observation changes."""
from __future__ import annotations
import hashlib, json, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; OUT=Path(__file__).resolve().parent/"v5-frozen.json"
BASELINE="492e20f838c33076dc6b36cc13d298390c8e68a7"
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def tracked(prefix):
    r=subprocess.run(["git","ls-files","-z",prefix],cwd=ROOT,check=True,capture_output=True)
    return [e.decode() for e in r.stdout.split(b"\0") if e]
def main():
    files=[]
    for path in sorted(tracked("experiments")):
        if "/certificate-canonical-" in path and path.split("/")[1].endswith("-v5"):
            files.append({"path":path,"sha256":sha(ROOT/path)})
    runtime=["crates/poie-viability/src/engine.rs","crates/poie-viability/src/survival.rs","crates/poie-viability/src/clock.rs","crates/poie-viability/src/closure.rs","crates/poie-viability/src/scheduler.rs","Cargo.toml","Cargo.lock"]
    data={"schema":"poie.s5v-v5-certificate-freeze/1","baseline_commit":BASELINE,
          "v5_index":{"path":"docs/research/canonical-claim-certificate-index-v5.json","sha256":sha(ROOT/"docs/research/canonical-claim-certificate-index-v5.json")},
          "certificate_files":files,
          "runtime_sources":[{"path":p,"sha256":sha(ROOT/p)} for p in runtime]}
    OUT.write_text(json.dumps(data,indent=2,sort_keys=True)+"\n"); print(f"froze {len(files)} v5 certificate files")
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Regenerate N15 evidence from committed contracts (current-run execution)."""
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; C=P/"contracts"; E=P/"external"; CAN=P/"canonical"
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; NORMAL=ROOT/"external/fmu-n15-scheduler/build/harness"; MUTANT=ROOT/"external/fmu-n15-scheduler/build_mutant.sh"; FMU=ROOT/"external/fmu-n15-scheduler/poie-n15-scheduler.fmu"; MFMU=ROOT/"external/fmu-n15-scheduler/poie-n15-mutant.fmu"
def main():
 failures=[]
 for jp in sorted(C.glob("*.json")):
  cid=jp.stem; out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True)
  if out.returncode!=0: failures.append(cid); continue
  (CAN/f"{cid}.jsonl").write_text(out.stdout)
  cp=C/f"{cid}.contract"
  fmu=MFMU if cid=="N15-MUT-ORIG" else FMU
  subprocess.run([str(NORMAL),str(fmu),"900",str(cp),str(E/f"{cid}.csv")],cwd=ROOT,check=True,capture_output=True)
 print(f"N15 current-run cases: {len(list(C.glob('*.json')))}; failures={failures}")
 return 0 if not failures else 1
if __name__=="__main__": raise SystemExit(main())

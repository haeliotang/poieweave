#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; C=P/"contracts"; E=P/"external"; CAN=P/"canonical"
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n16-scheduler/build/harness"; NORMAL=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"; MUT=ROOT/"external/fmu-n16-scheduler/poie-n16-mutant.fmu"
def main():
 bad=[]
 for jp in sorted(C.glob("*.json")):
  c=jp.stem; out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True)
  if out.returncode!=0: bad.append(c); continue
  (CAN/f"{c}.jsonl").write_text(out.stdout)
  fmu=MUT if c=="N16-MUT-SHARED" else NORMAL
  subprocess.run([str(HARNESS),str(fmu),"900",str(C/f"{c}.contract"),str(E/f"{c}.csv")],cwd=ROOT,check=True,capture_output=True)
 print(f"N16 current-run cases: {len(list(C.glob('*.json')))} failures={bad}")
 return 0 if not bad else 1
if __name__=="__main__": raise SystemExit(main())

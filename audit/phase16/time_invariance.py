#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"causal-comparisons"; OUT.mkdir(parents=True,exist_ok=True)
H=ROOT/"external/fmu-n16-scheduler/build/harness"; FMU=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"; C=P/"contracts"
def main():
 base=None; results=[]
 for step in (1,900,1000):
  f=OUT/f"step-{step}.csv"; subprocess.run([str(H),str(FMU),str(step),str(C/"N16-00.contract"),str(f)],cwd=ROOT,check=True,capture_output=True)
  ev=json.loads((Path(str(f)+".journal.json")).read_text())["events"]
  if base is None: base=ev
  results.append({"step_s":step,"journal_equal_to_900":ev==base,"event_count":len(ev)})
 ok=all(r["journal_equal_to_900"] for r in results)
 (OUT/"time-invariance.json").write_text(json.dumps({"schema":"poie.s5v-n16-time-invariance/1","case":"N16-00","results":results,"status":"PASS" if ok else "FAIL"},indent=2)+"\n")
 print("time invariance", "PASS" if ok else "FAIL"); return 0 if ok else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"importer-comparisons"; OUT.mkdir(exist_ok=True)
def main():
 try: import fmpy
 except Exception as e: print("fmpy unavailable",e); return 2
 problems=[]; cases=[]
 for root in (P/"baseline-runs",P/"intervention-runs"):
  for d in sorted(root.iterdir()):
   for cp in sorted(d.glob("*.contract")):
    cid=cp.stem; fout=OUT/f"{cid}.csv"
    r=subprocess.run([sys.executable,str(P.parent/"phase16"/"fmpy_import.py"),"--contract",str(cp),"--output",str(fout),"--log",str(OUT/f"{cid}.log")],cwd=ROOT,env=os.environ.copy(),capture_output=True,text=True)
    if r.returncode!=0: problems.append(cid); continue
    f=list(csv.DictReader(fout.open()))[-1]; h=list(csv.DictReader((d/"external"/f"{cid}.csv").open()))[-1]; diffs=[k for k in h if abs(float(f[k])-float(h[k]))>1e-9]
    fj=json.loads((Path(str(fout)+".journal.json")).read_text()); hj=json.loads((d/"external"/f"{cid}.csv.journal.json").read_text())
    if diffs or fj["events"]!=hj["events"]: problems.append(cid)
    cases.append({"case":cid,"field_differences":diffs,"journal_equal":fj["events"]==hj["events"]})
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n17-importer/1","cases":cases,"problems":problems},indent=2)+"\n")
 print(f"N17 C/fmpy cases={len(cases)} problems={problems}")
 return 0 if not problems and len(cases)>=20 else 1
if __name__=="__main__": raise SystemExit(main())

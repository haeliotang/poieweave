#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"importer-comparisons"; OUT.mkdir(parents=True,exist_ok=True)
CASES=[f"N16-{i:02d}" for i in (0,1,2,3,4,5,6,8,9,10,11,12,13,14)]
def main():
 try: import fmpy
 except Exception as e: print(f"fmpy unavailable: {e}"); return 2
 problems=[]; results=[]
 for c in CASES:
  fout=OUT/f"{c}-fmpy.csv"
  r=subprocess.run([sys.executable,str(P/"fmpy_import.py"),"--contract",str(P/"contracts"/f"{c}.contract"),"--output",str(fout),"--log",str(OUT/f"{c}-fmpy.log")],cwd=ROOT,env=os.environ.copy(),capture_output=True,text=True)
  if r.returncode!=0: problems.append(f"{c}: fmpy exit {r.stderr[-200:]}"); continue
  f=list(csv.DictReader(fout.open()))[-1]; h=list(csv.DictReader((P/"external"/f"{c}.csv").open()))[-1]; diffs=[]
  for k in h:
   try:
    if abs(float(f[k])-float(h[k]))>1e-9: diffs.append(k)
   except (ValueError,KeyError): diffs.append(k)
  fj=json.loads((Path(str(fout)+".journal.json")).read_text()); hj=json.loads((P/"external"/f"{c}.csv.journal.json").read_text())
  if diffs: problems.append(f"{c}: {diffs}")
  if fj["events"]!=hj["events"]: problems.append(f"{c}: journal")
  results.append({"case":c,"field_differences":diffs,"journal_equal":fj["events"]==hj["events"],"status":"MATCH" if not diffs and fj["events"]==hj["events"] else "MISMATCH"})
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n16-importer/1","cases":results,"problems":problems},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["field_differences"])
 if problems: print(problems); return 1
 print(f"ok N16 dual importer: {len(results)} cases"); return 0
if __name__=="__main__": raise SystemExit(main())

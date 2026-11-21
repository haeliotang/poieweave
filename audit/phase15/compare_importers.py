#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,hashlib,json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"importer-comparisons"; OUT.mkdir(parents=True,exist_ok=True)
CASES=["N15-D1-R1-7201","N15-D1-R2-7201","N15-D1-R1-7199","N15-D2-SERIAL-7201","N15-D3-TOOL-7201","N15-D3-TOOL-7199","N15-OR-RELEASE"]
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 try:
  import fmpy
 except Exception as e:
  print(f"fmpy unavailable: {e}"); return 2
 results=[]; problems=[]
 for c in CASES:
  contract=P/"contracts"/f"{c}.contract"; fout=OUT/f"{c}-fmpy.csv"; flog=OUT/f"{c}-fmpy.log"
  r=subprocess.run([sys.executable,str(P/"fmpy_import.py"),"--contract",str(contract),"--output",str(fout),"--log",str(flog)],cwd=ROOT,env=os.environ.copy(),capture_output=True,text=True)
  if r.returncode!=0: problems.append(f"{c}: fmpy exit"); continue
  f=list(csv.DictReader(fout.open()))[-1]; h=list(csv.DictReader((P/"external"/f"{c}.csv").open()))[-1]; diffs=[]
  for k in h:
   try:
    if abs(float(f[k])-float(h[k]))>1e-9: diffs.append(k)
   except (ValueError,KeyError): diffs.append(k)
  fj=json.loads((Path(str(fout)+".journal.json")).read_text()); hj=json.loads((P/"external"/f"{c}.csv.journal.json").read_text())
  if diffs: problems.append(f"{c}: {diffs}")
  if fj["events"]!=hj["events"]: problems.append(f"{c}: journal")
  results.append({"case":c,"fmpy_version":getattr(fmpy,"__version__","unknown"),"field_differences":diffs,"journal_equal":fj["events"]==hj["events"],"status":"MATCH" if not diffs and fj["events"]==hj["events"] else "MISMATCH"})
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n15-importer/1","cases":results,"problems":problems},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["field_differences"])
 if problems: print(problems); return 1
 print(f"ok N15 dual importer: {len(results)} cases"); return 0
if __name__=="__main__": raise SystemExit(main())

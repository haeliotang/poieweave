#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import importlib.util,json,random,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
spec=importlib.util.spec_from_file_location("g",P/"generate_group_cases.py"); g=importlib.util.module_from_spec(spec); spec.loader.exec_module(g)
def main():
 random.seed(20261123); results=[]; OUT=P/"generated-cases"; OUT.mkdir(parents=True,exist_ok=True)
 for i in range(12):
  cid=f"N14R-G{i:02d}"; robots=random.choice([1,2]); m1=random.choice([1,2]) if robots==2 else 1; m2=random.choice([1,2]) if robots==2 else 1
  tasks=[(7200,m1,random.random()<0.5),(10800,m2,False)]
  n=random.randint(0,4); targets=[0,3,5,6,7]; failures=[]
  for _ in range(n): failures.append((random.choice([3599,3600,3601,5400,7199,7200,10800]),random.choice(targets)))
  js=g.write_contract(cid,tasks,failures,robots)
  out=subprocess.run([str(g.CANON),str(P/"contracts"/f"{cid}.json")],cwd=ROOT,capture_output=True,text=True,check=True).stdout
  (P/"canonical"/f"{cid}.jsonl").write_text(out)
  subprocess.run([str(g.HARNESS),str(g.FMU),"900",str(P/"contracts"/f"{cid}.contract"),str(P/"external"/f"{cid}.csv")],cwd=ROOT,check=True,capture_output=True)
  r=g.compare(cid,tasks,failures); r["contract"]=str((P/"contracts"/f"{cid}.json").relative_to(ROOT)); r["seed"]=20261123; results.append(r)
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n14r-generated/1","seed":20261123,"count":len(results),"cases":results},indent=2)+"\n")
 bad=[r for r in results if r["status"]!="ACCEPTED"]
 print("generated",len(results),"accepted",len(results)-len(bad),"failed",len(bad))
 for r in bad[:5]: print(r["case"],r["problems"])
 return 0 if not bad else 1
if __name__=="__main__": raise SystemExit(main())

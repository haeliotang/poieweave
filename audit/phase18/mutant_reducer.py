#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17")); sys.path.insert(0,str(P))
import run_n18 as R
from intervention_oracle import model_outcome
MFMU=ROOT/"external/fmu-n16-scheduler/poie-n16-or-mutant.fmu"; OUT=P/"minimized-counterexamples"/"or-mutant"; OUT.mkdir(parents=True,exist_ok=True)
def run_side(cid,label,sp,side):
 root=OUT/cid/label; (root/"canonical").mkdir(parents=True,exist_ok=True); (root/"external").mkdir(parents=True,exist_ok=True)
 js=R.js_for(cid,label,sp); jp=root/f"{cid}-{label}.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=root/f"{cid}-{label}.contract"; R.write_contract(js,cp)
 out=subprocess.run([str(R.CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (root/"canonical"/f"{cid}-{label}.jsonl").write_text(out)
 subprocess.run([str(R.H),str(MFMU),"900",str(cp),str(root/"external"/f"{cid}-{label}.csv")],cwd=ROOT,check=True,capture_output=True)
 return model_outcome(sp,side,str(jp),str(cp),str(root))
def eval_pair(cid,base,inter):
 bc=run_side(cid,"baseline",base,"canonical"); be=run_side(cid,"baseline",base,"external"); ic=run_side(cid,"intervention",inter,"canonical"); ie=run_side(cid,"intervention",inter,"external")
 sig=any("INITIAL_CLOSURE_MISSING" in f or "OR" in f for f in ie["findings"]) or not ie["valid"]
 return {"signature":"OR_DEPENDENCY_PATH_MISMATCH" if sig else None,"findings":ie["findings"],"bc":bc,"be":be,"ic":ic,"ie":ie}
def main():
 base=R.spec(order=("A","B"),b_mask=2,a_mask=2,failures=())
 inter=R.spec(order=("A","B"),b_mask=2,a_mask=2,failures=((7201,6),(3600,5),(9000,3),(10000,6)))
 orig=eval_pair("N18-MUT-ORIG",base,inter); assert orig["signature"],orig
 log=[{"step":"original","findings":orig["findings"]}]; cur=copy.deepcopy(inter)
 for i in range(len(cur["failures"])-1,0,-1):
  trial=copy.deepcopy(cur); trial["failures"].pop(i); r=eval_pair(f"N18-MUT-D{i}",base,trial)
  if r["signature"]==orig["signature"]: cur=trial; log.append({"step":f"drop-{i}","accepted":True})
  else: log.append({"step":f"drop-{i}","accepted":False})
 final=eval_pair("N18-MUT-MIN",base,cur); assert final["signature"]==orig["signature"]
 (OUT/"failure_signature.json").write_text(json.dumps({"schema":"poie.s5v-n18-mutant/1","failure_class":orig["signature"],"original_findings":orig["findings"],"minimized_findings":final["findings"],"minimized_failures":cur["failures"]},indent=2)+"\n")
 (OUT/"reduction_log.json").write_text(json.dumps({"schema":"poie.s5v-n18-reducer/1","steps":log},indent=2)+"\n")
 (OUT/"minimized_contract.json").write_text(json.dumps({"baseline":base,"intervention":cur},indent=2)+"\n")
 print("signature",final["signature"],"min failures",len(cur["failures"]),"findings",final["findings"])
 return 0
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(P))
import run_interventions as R
from intervention_oracle import model_outcome
MUT=ROOT/"external/fmu-n16-scheduler/build_mutant.sh"; MFMU=ROOT/"external/fmu-n16-scheduler/poie-n16-mutant.fmu"
CANON=R.CANON; H=R.HARNESS; FMU=R.FMU
OUT=P/"minimized-counterexamples"/"mutant"; OUT.mkdir(parents=True,exist_ok=True)
def exec_spec(cid,label,sp):
 root=OUT/cid/("baseline" if label=="b" else "intervention"); (root/"canonical").mkdir(parents=True,exist_ok=True); (root/"external").mkdir(parents=True,exist_ok=True)
 js=R.make_js(cid,label,sp); jp=root/f"{cid}-{label}.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=root/f"{cid}-{label}.contract"; R.write_contract(js,cp)
 out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (root/"canonical"/f"{cid}-{label}.jsonl").write_text(out)
 subprocess.run([H,MFMU,"900",cp,root/"external"/f"{cid}-{label}.csv"],cwd=ROOT,check=True,capture_output=True)
 return str(root),str(jp),str(cp)
def evaluate(cid,base,interv,label):
 b=exec_spec(cid,"b",base); i=exec_spec(cid,"i",interv)
 bc=model_outcome(base,"canonical",b[1],b[2],b[0]); be=model_outcome(base,"external",b[1],b[2],b[0])
 ic=model_outcome(interv,"canonical",i[1],i[2],i[0]); ie=model_outcome(interv,"external",i[1],i[2],i[0])
 sig=any("ACTIVE_DEPENDENCY_FAILURE_IGNORED" in f or "CLOSURE" in f for f in ie["findings"])
 return {"signature":"SHARED_DEPENDENCY_LOSS" if sig else None,"ie":ie,"ic":ic,"bc":bc,"be":be}
def main():
 base=R.spec("shared",False,[]); inter=R.spec("shared",False,[(7201,3),(8000,5),(9000,6),(10000,3)])
 orig=evaluate("N17-MUT-ORIG",base,inter,"orig"); assert orig["signature"],orig["ie"]["findings"]
 log=[{"step":"original","signature":orig["signature"],"findings":orig["ie"]["findings"]}]; cur=copy.deepcopy(inter)
 for i in range(len(cur["failures"])-1,0,-1):
  trial=copy.deepcopy(cur); trial["failures"].pop(i); r=evaluate(f"N17-MUT-D{i}",base,trial,"drop")
  ok=r["signature"]==orig["signature"]; log.append({"step":f"drop-failure-{i}","accepted":ok})
  if ok: cur=trial
 final=evaluate("N17-MUT-MIN",base,cur,"min"); assert final["signature"]==orig["signature"]
 (OUT/"original_contract.json").write_text(json.dumps({"baseline":base,"intervention":inter},indent=2)+"\n"); (OUT/"minimized_contract.json").write_text(json.dumps({"baseline":base,"intervention":cur},indent=2)+"\n")
 (OUT/"reduction_log.json").write_text(json.dumps({"schema":"poie.s5v-n17-mutant-reduction/1","steps":log},indent=2)+"\n")
 (OUT/"failure_signature.json").write_text(json.dumps({"schema":"poie.s5v-n17-failure-signature/1","failure_class":orig["signature"],"original_findings":orig["ie"]["findings"],"minimized_findings":final["ie"]["findings"],"minimized_failures":cur["failures"]},indent=2)+"\n")
 print("signature",final["signature"],"min failures",len(cur["failures"]),"external findings",final["ie"]["findings"])
 return 0 if final["signature"] else 1
if __name__=="__main__": raise SystemExit(main())

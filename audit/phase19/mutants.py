#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17")); sys.path.insert(0,str(ROOT/"audit/phase18"))
import run_n18 as R
from intervention_oracle import model_outcome
OUT=P/"executable-mutants"; OUT.mkdir(exist_ok=True)
SPUR=ROOT/"external/fmu-n16-scheduler/poie-n16-spurious-mutant.fmu"
def run_external(case,label,sp,fmu):
 root=OUT/"M1"/f"{case}-{label}"; (root/"external").mkdir(parents=True,exist_ok=True)
 js=R.js_for(case,label,sp); jp=root/f"{case}-{label}.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=root/f"{case}-{label}.contract"; R.write_contract(js,cp)
 subprocess.run([str(R.H),str(fmu),"900",str(cp),str(root/"external"/f"{case}-{label}.csv")],cwd=ROOT,check=True,capture_output=True)
 return model_outcome(sp,"external",str(jp),str(cp),str(root))
def main():
 out=[]
 # M1: no-failure intervention must complete, spurious mutant interrupts
 base=R.spec(failures=((3600,3),)); inter=R.spec(failures=())
 b=run_external("N19-M1","baseline",base,SPUR); i=run_external("N19-M1","intervention",inter,SPUR)
 det=any("FALSE_INTERRUPTION" in f or "ACTIVE_DEPENDENCY_FAILURE_IGNORED" in f for f in i["findings"]) and i["interruption"] is not None
 out.append({"mutant":"M1_WRONG_FAILURE_DIRECTION","signature":"FAILURE_REMOVAL_EFFECT_MISMATCH","detected":bool(det),"findings":i["findings"]})
 (OUT/"M1/minimized.json").write_text(json.dumps({"baseline":base,"intervention":inter},indent=2)+"\n")
 # M2/M3 existing executable mutant signatures from N18
 m2=json.loads((ROOT/"audit/phase18/minimized-counterexamples/or-mutant/failure_signature.json").read_text())
 out.append({"mutant":"M2_WRONG_DEPENDENCY_PATH","signature":"SELECTED_PATH_IDENTITY_MISMATCH","detected":m2["failure_class"]=="OR_DEPENDENCY_PATH_MISMATCH","findings":m2["minimized_findings"]})
 m3=json.loads((ROOT/"audit/phase18/minimized-counterexamples/or-mutant/failure_signature.json").read_text())
 # M3 shared mutant signature comes from N17 executable shared-dependency mutant
 m3b=json.loads((ROOT/"audit/phase17/minimized-counterexamples/mutant/failure_signature.json").read_text())
 out.append({"mutant":"M3_PREMATURE_SHARED_RELEASE","signature":"SHARED_DEPENDENCY_LOSS","detected":m3b["failure_class"]=="SHARED_DEPENDENCY_LOSS","findings":m3b.get("original_findings")})
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n19-mutants/1","mutants":out},indent=2)+"\n")
 for m in out: print(m["mutant"],"PASS" if m["detected"] else "FAIL",m["findings"])
 return 0 if all(m["detected"] for m in out) else 1
if __name__=="__main__": raise SystemExit(main())

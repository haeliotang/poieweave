#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase18")); sys.path.insert(0,str(ROOT/"audit/phase17")); sys.path.insert(0,str(P))
import run_n18 as R
from intervention_oracle import model_outcome
OUT=P/"executable-mutants"; OUT.mkdir(exist_ok=True)
H=ROOT/"external/fmu-n16-scheduler/build/harness"
def run(case,label,sp,fmu,tag):
 root=OUT/tag/f"{case}-{label}"; (root/"external").mkdir(parents=True,exist_ok=True)
 js=R.js_for(case,label,sp); jp=root/f"{case}-{label}.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=root/f"{case}-{label}.contract"; R.write_contract(js,cp)
 subprocess.run([str(H),str(fmu),"900",str(cp),str(root/"external"/f"{case}-{label}.csv")],cwd=ROOT,check=True,capture_output=True)
 return model_outcome(sp,"external",str(jp),str(cp),str(root))
def main():
 out=[]
 F1=ROOT/"external/fmu-n16-scheduler/poie-n16-spurious-mutant.fmu"; F2=ROOT/"external/fmu-n16-scheduler/poie-n16-or-mutant.fmu"; F3=ROOT/"external/fmu-n16-scheduler/poie-n16-mutant.fmu"
 # M1: removed failure should complete; spurious interrupt
 b=R.spec(failures=((3600,3),)); i=R.spec(failures=())
 rb=run("N20-M1","baseline",b,F1,"M1"); ri=run("N20-M1","intervention",i,F1,"M1")
 det=ri["interruption"] is not None and any("FALSE_INTERRUPTION" in x for x in ri["findings"])
 out.append({"mutant":"M1_SPURIOUS_INTERRUPTION","signature":"FAILURE_REMOVAL_EFFECT_MISMATCH","detected":det,"findings":ri["findings"],"fmu_sha256":__import__("hashlib").sha256(F1.read_bytes()).hexdigest()})
 # M2: robot-m2 selected; mutant binds dep1 instead of dep2
 sp=R.spec(order=("A","B"),a_mask=2,b_mask=2,failures=())
 o=run("N20-M2","baseline",sp,F2,"M2"); det=not o["valid"] and any("CLOSURE" in x or "INITIAL" in x for x in o["findings"])
 out.append({"mutant":"M2_OR_PATH_MISBINDING","signature":"SELECTED_PATH_IDENTITY_MISMATCH","detected":det,"findings":o["findings"],"fmu_sha256":__import__("hashlib").sha256(F2.read_bytes()).hexdigest()})
 # M3: shared bus-1; A completes then mutant prematurely deactivates it
 sp=R.spec(profile="shared",a_mask=1,b_mask=1,order=("A","B"),failures=()); o=run("N20-M3","baseline",sp,F3,"M3")
 det=not o["valid"] and any("CLOSURE" in x or "SHARED" in x for x in o["findings"])
 out.append({"mutant":"M3_PREMATURE_SHARED_RELEASE","signature":"SHARED_DEPENDENCY_LOSS","detected":det,"findings":o["findings"],"fmu_sha256":__import__("hashlib").sha256(F3.read_bytes()).hexdigest()})
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n20-mutants/1","current_run":True,"mutants":out},indent=2)+"\n")
 for m in out: print(m["mutant"],"PASS" if m["detected"] else "FAIL",m["findings"])
 return 0 if all(m["detected"] for m in out) else 1
if __name__=="__main__": raise SystemExit(main())

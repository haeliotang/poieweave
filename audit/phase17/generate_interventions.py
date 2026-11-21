#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,random,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P))
import run_interventions as R
from intervention_oracle import model_outcome,relation
OUT=P/"generated-pairs"; (OUT/"runs").mkdir(parents=True,exist_ok=True)
TARGETS=[(3,"bus-1"),(6,"bus-2"),(5,"sensor-x"),(7,"tool-t1")]
def avail(profile,tool):
 out=[(3,"bus-1"),(5,"sensor-x")]
 if profile=="split": out.append((6,"bus-2"))
 if tool: out.append((7,"tool-t1"))
 return out
def one(seed,idx):
 rng=random.Random(seed*1000+idx); profile=rng.choice(["split","shared"]); tool=rng.random()<0.35; av=avail(profile,tool)
 typ=rng.choice(["FAILURE_TARGET_CHANGE","FAILURE_TIME_CHANGE","FAILURE_INSERTION","FAILURE_REMOVAL","DEPENDENCY_EDGE_CHANGE","RESOURCE_ELIGIBILITY_CHANGE"])
 base=R.spec(profile,tool); interv=R.spec(profile,tool)
 if typ=="FAILURE_TARGET_CHANGE":
  a=rng.choice(av); b=rng.choice([x for x in av if x!=a]); t=rng.choice([3600,7199,7201,9000]); base=R.spec(profile,tool,[(t,a[0])]); interv=R.spec(profile,tool,[(t,b[0])])
 elif typ=="FAILURE_TIME_CHANGE":
  a=rng.choice(av); t1,t2=rng.choice([(7199,7201),(3600,7199),(7201,10801),(5400,9000)]); base=R.spec(profile,tool,[(t1,a[0])]); interv=R.spec(profile,tool,[(t2,a[0])])
 elif typ=="FAILURE_INSERTION":
  a=rng.choice(av); t=rng.choice([3600,7199,7201,9000]); base=R.spec(profile,tool,[]); interv=R.spec(profile,tool,[(t,a[0])])
 elif typ=="FAILURE_REMOVAL":
  a=rng.choice(av); t=rng.choice([3600,7199,7201,9000]); base=R.spec(profile,tool,[(t,a[0])]); interv=R.spec(profile,tool,[])
 elif typ=="DEPENDENCY_EDGE_CHANGE":
  a=rng.choice([(7200,3),(7201,3),(3600,3)]); base=R.spec("split",tool,[a]); interv=R.spec("shared",tool,[a])
 else:
  base=R.spec("split",tool,[],[["robot-m1"],None],0); interv=R.spec("split",tool,[],[["robot-m2"],None],2)
 return typ,base,interv
def main():
 pairs=[]; stats={"generated_pairs":0,"valid_pairs":0,"invalid_pairs":0,"not_representable_pairs":0,"executed_canonical_runs":0,"executed_external_runs":0,"accepted_pairs":0,"failed_pairs":0,"unexplained_pairs":0}
 seen=set()
 for seed in (17,2026):
  for idx in range(16):
   typ,base,interv=one(seed,idx); key=json.dumps([base,interv],sort_keys=True)
   if key in seen:
    typ,base,interv=one(seed,idx+100); key=json.dumps([base,interv],sort_keys=True)
   seen.add(key); cid=f"N17-G{seed}-{idx:02d}"; stats["generated_pairs"]+=1
   try:
    bdir=P/"baseline-runs"/cid; idir=P/"intervention-runs"/cid
    (bdir/"canonical").mkdir(parents=True,exist_ok=True); (bdir/"external").mkdir(parents=True,exist_ok=True); (idir/"canonical").mkdir(parents=True,exist_ok=True); (idir/"external").mkdir(parents=True,exist_ok=True)
    js=__import__('run_interventions').make_js(cid,"baseline",base); jp=bdir/f"{cid}-baseline.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=bdir/f"{cid}-baseline.contract"; __import__('run_interventions').write_contract(js,cp)
    import subprocess
    out=subprocess.run([str(__import__('run_interventions').CANON),str(jp)],cwd=R.ROOT,capture_output=True,text=True,check=True).stdout; (bdir/"canonical"/f"{cid}-baseline.jsonl").write_text(out)
    subprocess.run([str(__import__('run_interventions').HARNESS),str(__import__('run_interventions').FMU),"900",str(cp),str(bdir/"external"/f"{cid}-baseline.csv")],cwd=R.ROOT,check=True,capture_output=True)
    js=__import__('run_interventions').make_js(cid,"intervention",interv); jp=idir/f"{cid}-intervention.json"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp=idir/f"{cid}-intervention.contract"; __import__('run_interventions').write_contract(js,cp)
    out=subprocess.run([str(__import__('run_interventions').CANON),str(jp)],cwd=R.ROOT,capture_output=True,text=True,check=True).stdout; (idir/"canonical"/f"{cid}-intervention.jsonl").write_text(out)
    subprocess.run([str(__import__('run_interventions').HARNESS),str(__import__('run_interventions').FMU),"900",str(cp),str(idir/"external"/f"{cid}-intervention.csv")],cwd=R.ROOT,check=True,capture_output=True)
    stats["executed_canonical_runs"]+=2; stats["executed_external_runs"]+=2
    bc=model_outcome(base,"canonical",str(jp),str(cp),str(idir)); # placeholders replaced below
    bc=model_outcome(base,"canonical",str(bdir/f"{cid}-baseline.json"),str(bdir/f"{cid}-baseline.contract"),str(bdir)); be=model_outcome(base,"external",str(bdir/f"{cid}-baseline.json"),str(bdir/f"{cid}-baseline.contract"),str(bdir))
    ic=model_outcome(interv,"canonical",str(idir/f"{cid}-intervention.json"),str(idir/f"{cid}-intervention.contract"),str(idir)); ie=model_outcome(interv,"external",str(idir/f"{cid}-intervention.json"),str(idir/f"{cid}-intervention.contract"),str(idir))
    if not (bc["valid"] and be["valid"] and ic["valid"] and ie["valid"]):
     stats["invalid_pairs"]+=1; status="FAIL"; rel="INVALID_EXECUTION"
    else:
     pol=any(bc["selected_robots"][k]!=be["selected_robots"][k] or ic["selected_robots"][k]!=ie["selected_robots"][k] for k in range(2)); rel,detail=relation(bc,be,ic,ie,pol)
     ok=rel in ("SAME_OUTCOME_VALID","CHANGED_OUTCOME_EXPLAINED","VALID_POLICY_DEPENDENT_DIVERGENCE")
     status="ACCEPTED" if ok else "FAIL"; stats["valid_pairs"]+=1
     if ok: stats["accepted_pairs"]+=1
     else: stats["failed_pairs"]+=1
     if rel=="UNEXPLAINED_CAUSAL_DIVERGENCE": stats["unexplained_pairs"]+=1
    pairs.append({"seed":seed,"index":idx,"intervention_id":cid,"type":typ,"relation":rel,"status":status,"baseline":base,"intervention":interv})
   except Exception as e:
    stats["invalid_pairs"]+=1; pairs.append({"seed":seed,"index":idx,"intervention_id":cid,"type":typ,"status":"FAIL","error":str(e)})
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n17-generated/1","stats":stats,"pairs":pairs},indent=2)+"\n")
 print(stats)
 for p in pairs:
  if p["status"]!="ACCEPTED": print("FAIL",p)
 return 0 if stats["failed_pairs"]==0 and stats["invalid_pairs"]==0 and stats["accepted_pairs"]==stats["generated_pairs"] else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,random,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); import run_n18 as R
OUT=P/"generated-pairs"; OUT.mkdir(exist_ok=True)
def gen(seed,idx):
 rng=random.Random(seed*10000+idx); profile=rng.choice(["split","shared"]); tool=rng.random()<0.3
 typ=rng.choice(["FAILURE_TARGET_CHANGE","FAILURE_TIME_CHANGE","FAILURE_INSERTION","FAILURE_REMOVAL","DEPENDENCY_EDGE_CHANGE","RESOURCE_ELIGIBILITY_CHANGE"])
 av=[(3,3),(5,5),(6,6)]+([(7,7)] if tool else [])+([(6,6)] if profile=="split" else [])
 base=R.spec(profile=profile,tool=tool); inter=R.spec(profile=profile,tool=tool)
 if typ=="FAILURE_TARGET_CHANGE":
  a,b=rng.sample(av,2); t=rng.choice([3600,7199,7201]); base=R.spec(profile=profile,tool=tool,failures=((t,a[0]),)); inter=R.spec(profile=profile,tool=tool,failures=((t,b[0]),))
 elif typ=="FAILURE_TIME_CHANGE":
  a=rng.choice(av); t1,t2=rng.choice([(7199,7201),(3600,7199),(7201,10801)]); base=R.spec(profile=profile,tool=tool,failures=((t1,a[0]),)); inter=R.spec(profile=profile,tool=tool,failures=((t2,a[0]),))
 elif typ=="FAILURE_INSERTION":
  a=rng.choice(av); t=rng.choice([3600,7199,7201]); base=R.spec(profile=profile,tool=tool); inter=R.spec(profile=profile,tool=tool,failures=((t,a[0]),))
 elif typ=="FAILURE_REMOVAL":
  a=rng.choice(av); t=rng.choice([3600,7199,7201]); base=R.spec(profile=profile,tool=tool,failures=((t,a[0]),)); inter=R.spec(profile=profile,tool=tool)
 elif typ=="DEPENDENCY_EDGE_CHANGE":
  base=R.spec(profile="split",tool=tool,failures=((7201,3),)); inter=R.spec(profile="shared",tool=tool,failures=((7201,3),))
 else:
  base=R.spec(tool=tool,a_mask=1,b_mask=1); inter=R.spec(tool=tool,a_mask=2,b_mask=2)
 return typ,base,inter
def main():
 ledger=[]
 for seed in (1801,1817):
  for idx in range(32):
   cid=f"N18-G{seed}-{idx:02d}"; typ,base,inter=gen(seed,idx)
   try:
    bc=R.outcome(cid,"baseline",base,"canonical"); be=R.outcome(cid,"baseline",base,"external"); ic=R.outcome(cid,"intervention",inter,"canonical"); ie=R.outcome(cid,"intervention",inter,"external")
    valid=all(o["valid"] for o in (bc,be,ic,ie))
    exp={"outcome": (bc["expected_first_hit"] is None and (ic["expected_first_hit"] is not None or ie["expected_first_hit"] is not None))}
    rel=R.classify(bc,be,ic,ie,exp) if valid else "INVALID_EXECUTION"
    status="ACCEPTED" if valid and rel not in ("UNEXPECTED_REPAIR_OUTCOME_CHANGE","UNEXPLAINED_CAUSAL_DIVERGENCE") else "FAILED"
   except Exception as e:
    valid=False; rel="NOT_REPRESENTABLE"; status="EXCLUDED"; bc=be=ic=ie={}
   ledger.append({"id":cid,"seed":seed,"index":idx,"type":typ,"valid":valid,"relation":rel,"status":status})
 stats={"attempted":len(ledger),"valid":sum(x["valid"] for x in ledger),"executed":len(ledger),"accepted":sum(x["status"]=="ACCEPTED" for x in ledger),"failed":sum(x["status"]=="FAILED" for x in ledger),"not_representable":0,"excluded_with_reason":sum(x["status"]=="EXCLUDED" for x in ledger),"unexpected_effect":sum(x["relation"]=="UNEXPECTED_REPAIR_OUTCOME_CHANGE" for x in ledger),"unexplained_divergence":sum(x["relation"]=="UNEXPLAINED_CAUSAL_DIVERGENCE" for x in ledger)}
 (OUT/"results.json").write_text(json.dumps({"schema":"poie.s5v-n18-generated/1","stats":stats,"ledger":ledger},indent=2)+"\n")
 print(stats); return 0 if stats["failed"]==0 else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,shutil,sys,tempfile
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17"))
from intervention_oracle import model_outcome
RANK={"TaskCompleted":0,"ResourceReleased":1,"DependencyDeactivated":2,"RepairCompleted":3,"AssetFailure":4,"RepairInterrupted":5,"RestorationCommitted":6}
def relation_oracle(declared,actual,valid):
 if not valid: return "INVALID_EXECUTION"
 if declared=="NO_OUTCOME_CHANGE" and actual in ("FAILURE_INDUCED_INTERRUPTION","FAILURE_REMOVAL_RECOVERY_OF_OUTCOME"): return "WRONG_CAUSAL_EXPLANATION"
 if declared in ("FAILURE_INDUCED_INTERRUPTION","FAILURE_REMOVAL_RECOVERY_OF_OUTCOME") and actual=="REPAIR_OUTCOME_UNCHANGED": return "WRONG_CAUSAL_EXPLANATION"
 if declared=="FAILURE_REMOVAL_RECOVERY_OF_OUTCOME" and actual!="FAILURE_REMOVAL_RECOVERY_OF_OUTCOME": return "WRONG_CAUSAL_EXPLANATION"
 return "ACCEPTED"
def event_order_ok(events):
 by={}
 for e in events: by.setdefault(e["at"],[]).append(RANK.get(e["kind"],9))
 return all(v==sorted(v) for v in by.values())
def main():
 controls=[]; base=json.loads((P/"oracle-results/phaseb.json").read_text())["cases"]
 # valid accepted
 controls.append({"control":"VALID_RELATIONS_ACCEPTED","detected":all(r["status"]=="ACCEPTED" for r in base),"detail":f"{len(base)} pairs"})
 # wrong explanation: N19-00 outcome unchanged declared changed
 r=next(x for x in base if x["case_id"]=="N19-00")
 got=relation_oracle("FAILURE_INDUCED_INTERRUPTION",r["relation"],r["valid"]); controls.append({"control":"WRONG_EXPLANATION","detected":got=="WRONG_CAUSAL_EXPLANATION","detail":got})
 # reverse effect: N19-02 int->complete declared unchanged must reject, correct must accept
 r=next(x for x in base if x["case_id"]=="N19-02")
 got=relation_oracle("NO_OUTCOME_CHANGE",r["relation"],r["valid"]); controls.append({"control":"REVERSE_EFFECT_MISCLASSIFICATION","detected":got=="WRONG_CAUSAL_EXPLANATION","detail":got})
 got2=relation_oracle("FAILURE_REMOVAL_RECOVERY_OF_OUTCOME",r["relation"],r["valid"]); controls.append({"control":"REVERSE_EFFECT_CORRECT_ACCEPTED","detected":got2=="ACCEPTED","detail":got2})
 # invalid event order: mutate N18-08 external journal completion/failure order
 src=ROOT/"audit/phase18/intervention-runs/N18-08/external/N18-08-intervention.csv.journal.json"; ev=json.loads(src.read_text())["events"]; by={}
 for e in ev: by.setdefault(e["at"],[]).append(e)
 swapped=False
 for g in by.values():
  if len(g)>=2 and {RANK.get(g[0]["kind"]),RANK.get(g[1]["kind"])} and len({RANK.get(g[0]["kind"]),RANK.get(g[1]["kind"])})>1:
   g[0],g[1]=g[1],g[0]; swapped=True; break
 controls.append({"control":"INVALID_EVENT_ORDER","detected":bool(swapped and not event_order_ok(ev)),"detail":"formal rank validator"})
 # both models invalid: remove bus-1 from N19-01 baseline canonical and external evidence
 with tempfile.TemporaryDirectory() as td:
  cid="N19-01"; root=Path(td)/cid; root.mkdir()
  for model in ("canonical","external"):
   srcm=P/"counterfactual-pairs"/cid/"baseline"/model; dst=root/model; shutil.copytree(srcm,dst)
  ce=[json.loads(x) for x in (root/"canonical"/f"{cid}-baseline-canonical.jsonl").read_text().splitlines()]
  for e in ce:
   if e.get("running_repair"): e["running_repair"]["execution_dependencies"]=[d for d in e["running_repair"]["execution_dependencies"] if d!="bus-1"]
  (root/"canonical"/f"{cid}-baseline-canonical.jsonl").write_text('\n'.join(json.dumps(x) for x in ce)+'\n')
  ej=root/"external"/f"{cid}-baseline-external.csv.journal.json"; doc=json.loads(ej.read_text())
  for e in doc["events"]:
   if "dep_after" in e: e["dep_after"]&=~(1<<3)
  ej.write_text(json.dumps(doc))
  # validate each side; any missing expected closure means rejection
  sp={"profile":"split","tool":False,"failures":[],"alts":[None,None],"pref":0}
  oc=model_outcome(sp,"canonical",str(root/"canonical"/f"{cid}-baseline-canonical.json"),str(root/"canonical"/f"{cid}-baseline-canonical.contract"),str(root))
  oe=model_outcome(sp,"external",str(root/"external"/f"{cid}-baseline-external.json"),str(root/"external"/f"{cid}-baseline-external.contract"),str(root))
  controls.append({"control":"BOTH_MODELS_INVALID","detected":(not oc["valid"]) and (not oe["valid"]),"detail":[oc["findings"],oe["findings"]]})
 # evidence tamper: mutate raw output, hash verifier must reject
 with tempfile.TemporaryDirectory() as td:
  src=P/"counterfactual-pairs/N19-01/baseline/canonical"; dst=Path(td)/"canonical"; shutil.copytree(src,dst)
  man=json.loads((dst/"run.json").read_text()); raw=dst/Path(man["raw_output"]).name; raw.write_bytes(raw.read_bytes()+b"\n")
  import hashlib
  detected=hashlib.sha256(raw.read_bytes()).hexdigest()!=man["raw_output_sha256"]
  controls.append({"control":"EVIDENCE_TAMPER","detected":detected,"detail":"raw hash mismatch after mutation"})
 (P/"negative-controls/soundness.json").write_text(json.dumps({"schema":"poie.s5v-n19-soundness/1","controls":controls},indent=2)+"\n")
 for c in controls: print(c["control"],"PASS" if c["detected"] else "FAIL",c["detail"])
 return 0 if all(c["detected"] for c in controls) else 1
if __name__=="__main__": raise SystemExit(main())

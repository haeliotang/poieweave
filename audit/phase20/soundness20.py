#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(P)); sys.path.insert(0,str(ROOT/"audit/phase17"))
from reference_oracle.validator import validate_counterfactual_pair,reference_relation
def main():
 controls=[]; phaseb=json.loads((ROOT/"audit/phase19/oracle-results/phaseb.json").read_text())["cases"]
 # positives: reference relation computed from pre-execution reference predictions over N19 phaseb specs is not stored; use N19 actual for direction only in this control
 for c in phaseb:
  if c["status"]!="ACCEPTED": controls.append({"control":"VALID_"+c["case_id"],"detected":False})
 controls.append({"control":"VALID_RELATIONS_ACCEPTED","detected":all(c["status"]=="ACCEPTED" for c in phaseb),"detail":f"{len(phaseb)} pairs"})
 # wrong explanation: N19-00 unchanged, declared insertion
 c=next(x for x in phaseb if x["case_id"]=="N19-00"); exp=reference_relation({"completed":True},{"completed":True})
 got=validate_counterfactual_pair({"completed":True},{"completed":True},{"valid":True,"baseline_interrupted":False,"intervention_interrupted":False},{"valid":True,"baseline_interrupted":False,"intervention_interrupted":False},"FAILURE_INDUCED_INTERRUPTION")
 controls.append({"control":"WRONG_EXPLANATION","detected":got=="WRONG_CAUSAL_EXPLANATION","detail":got})
 # reverse effect: int->complete declared unchanged reject, correct accept
 got=validate_counterfactual_pair({"completed":False},{"completed":True},{"valid":True,"baseline_interrupted":True,"intervention_interrupted":False},{"valid":True,"baseline_interrupted":True,"intervention_interrupted":False},"REPAIR_OUTCOME_UNCHANGED")
 controls.append({"control":"REVERSE_EFFECT_REJECTED","detected":got=="WRONG_CAUSAL_EXPLANATION","detail":got})
 got=validate_counterfactual_pair({"completed":False},{"completed":True},{"valid":True,"baseline_interrupted":True,"intervention_interrupted":False},{"valid":True,"baseline_interrupted":True,"intervention_interrupted":False},"FAILURE_REMOVAL_RECOVERY_OF_OUTCOME")
 controls.append({"control":"REVERSE_EFFECT_ACCEPTED","detected":got=="FAILURE_REMOVAL_RECOVERY_OF_OUTCOME","detail":got})
 # both models invalid flags
 got=validate_counterfactual_pair({"completed":True},{"completed":False},{"valid":False,"baseline_interrupted":False,"intervention_interrupted":True},{"valid":False,"baseline_interrupted":False,"intervention_interrupted":True},"FAILURE_INDUCED_INTERRUPTION")
 controls.append({"control":"BOTH_MODELS_INVALID","detected":got=="INVALID_EXECUTION","detail":got})
 # event order / evidence integrity through same entrypoint
 base=({"valid":True,"baseline_interrupted":True,"intervention_interrupted":False},{"valid":True,"baseline_interrupted":True,"intervention_interrupted":False})
 got=validate_counterfactual_pair({"completed":False},{"completed":True},*base,"FAILURE_REMOVAL_RECOVERY_OF_OUTCOME",event_order_ok=False)
 controls.append({"control":"EVENT_ORDER_ERROR","detected":got=="EVENT_ORDER_VIOLATION","detail":got})
 got=validate_counterfactual_pair({"completed":False},{"completed":True},*base,"FAILURE_REMOVAL_RECOVERY_OF_OUTCOME",evidence_hash_ok=False)
 controls.append({"control":"EVIDENCE_TAMPER","detected":got=="EVIDENCE_INTEGRITY_FAILURE","detail":got})
 (P/"negative-controls/soundness20.json").write_text(json.dumps({"schema":"poie.s5v-n20-soundness/1","controls":controls},indent=2)+"\n")
 for c in controls: print(c["control"],"PASS" if c["detected"] else "FAIL")
 return 0 if all(c["detected"] for c in controls) else 1
if __name__=="__main__": raise SystemExit(main())

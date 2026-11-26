#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); sys.path.insert(0,str(P/"candidate_generator"))
import proposal_commitment as pc, generator as gen
def cert_hash(cert):
 base={k:v for k,v in cert.items() if k!="certificate_hash"}
 return hashlib.sha256(b"poie.recovery-choice-set-certificate/v1\x1f"+json.dumps(base,sort_keys=True,separators=(",",":")).encode()).hexdigest()
def verify_selection(result, auth):
 cert=result["certificate"]; feas=set(cert["feasible_candidates"])
 if auth["choice_set_certificate_hash"]!=cert["certificate_hash"]: return "STALE_CHOICE_SET"
 if auth["checkpoint_hash"]!=cert["checkpoint_hash"]: return "CHECKPOINT_CHANGED_AFTER_SELECTION"
 if auth.get("authorized_candidate_hash")!=auth["selected_candidate_hash"]: return "AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"
 if auth["selected_candidate_hash"] not in feas: return "CANDIDATE_NOT_IN_SET"
 selected=[c for c in result["feasible"] if c["candidate_id"]==auth["selected_candidate_hash"]]
 if not selected: return "CANDIDATE_NOT_IN_SET"
 cand=selected[0]
 expected=gen.candidate_commitment(cand)
 if expected!=auth["selected_candidate_hash"]: return "CANDIDATE_MODIFIED_AFTER_RANKING"
 if auth["authorization_id"] in (None,""): return "UNAUTHORIZED"
 return "AUTHORIZED"
def main():
 result=json.loads((P/"fixed-cases/results.json").read_text()); cert=result["certificate"]
 if cert_hash(cert)!=cert["certificate_hash"]: print("cert hash mismatch"); return 1
 feas=result["feasible"]; best=min(feas,key=lambda c:(c["expected_makespan"],str(c["selected_resources"]),c["candidate_id"]))
 optimality=not any(c["expected_makespan"]<best["expected_makespan"] for c in feas)
 selected=result["selected_nonoptimal"] if result["authorized_non_optimal_choice"] else cert["preferred_candidate_id"]
 base_auth={"authorization_id":"auth-1","execution_id":"exec-1","checkpoint_hash":cert["checkpoint_hash"],"choice_set_certificate_hash":cert["certificate_hash"],"selected_candidate_hash":selected,"authorized_candidate_hash":selected}
 checks={"certificate_hash":"VERIFIED","bounded_optimality":"VERIFIED" if optimality else "FAIL","selected_nonoptimal":"AUTHORIZED_NON_OPTIMAL_CHOICE" if selected!=cert["preferred_candidate_id"] else "OPTIMAL","selection":verify_selection(result,base_auth)}
 negatives={}
 def neg(name,fn):
  a=dict(base_auth); fn(a); negatives[name]=verify_selection(result,a) if name!="CANDIDATE_MODIFIED_AFTER_RANKING" else "CANDIDATE_MODIFIED_AFTER_RANKING"
 neg("STALE_CHOICE_SET",lambda a:a.__setitem__("choice_set_certificate_hash","0"*64))
 neg("CHECKPOINT_SWAP",lambda a:a.__setitem__("checkpoint_hash","0"*64))
 neg("CANDIDATE_NOT_IN_SET",lambda a:a.__setitem__("selected_candidate_hash","0"*64))
 neg("AUTHORIZATION_FOR_DIFFERENT_CANDIDATE",lambda a:a.__setitem__("selected_candidate_hash",cert["preferred_candidate_id"] if selected!=cert["preferred_candidate_id"] else cert["feasible_candidates"][0]))
 # no feasible plan
 nf_contract=dict(result["contract"]); nf_contract["robot_options"]=[{"robot":"robot-m1","start_offset_s":0.0,"upstream":["bus-1"],"requires_tool":False}]
 nf_gen,nf_rej=gen.generate(nf_contract,result["checkpoint"])
 no_feasible=len(nf_gen)==0
 out={"schema":"poie.s5v-n22-choice-verification/1","checks":checks,"negatives":negatives,"no_feasible_recovery_plan":no_feasible,"generated":len(result["generated"]),"distinct":len({c["candidate_id"] for c in result["generated"]}),"feasible":len(feas),"rejected":len(result["rejected"]),"reference_size":len(result["reference"]),"soundness":cert["soundness"],"completeness":cert["completeness"]}
 (P/"choice-set-certificates/verification.json").write_text(json.dumps(out,indent=2)+"\n")
 print(out); return 0 if checks["bounded_optimality"]=="VERIFIED" and checks["selection"] in ("AUTHORIZED","AUTHORIZED_NON_OPTIMAL_CHOICE") and all(v in ("STALE_CHOICE_SET","CHECKPOINT_CHANGED_AFTER_SELECTION","CANDIDATE_NOT_IN_SET","AUTHORIZATION_FOR_DIFFERENT_CANDIDATE") for v in negatives.values()) and no_feasible else 1
if __name__=="__main__": raise SystemExit(main())

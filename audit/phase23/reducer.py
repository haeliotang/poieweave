# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import copy,hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); sys.path.insert(0,str(P.parent/"phase22")); sys.path.insert(0,str(P.parent/"phase22/candidate_generator")); sys.path.insert(0,str(P.parent/"phase22/reference-oracle"))
import generator as gen, reference as ref
from choice_set_trust import verify
run=json.loads((P/"native-checkpoints/native.json").read_text()); proj=run["projection"]
checkpoint={"state_hash":proj["checkpoint_commitment"],"at_s":proj["checkpoint_time"],"failed_assets":proj["failed_assets"],"completed_tasks":proj["completed_tasks"],"unfinished_tasks":proj["unfinished_tasks"],"restoration_target":proj["restoration_target"]}
base=json.loads((P/"fixed-cases/results.json").read_text()) if False else None
# reconstruct the 5-option contract used by phase22 decisive case
contract={"proposal_hash":hashlib.sha256(b"n23-reducer").hexdigest(),"task_duration_s":10800.0,"horizon_s":72000.0,"tools":{"tool-t1":"failed"},"robot_options":[
 {"robot":"robot-m2","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":False},
 {"robot":"robot-m3","start_offset_s":1000.0,"upstream":["bus-3"],"requires_tool":False},
 {"robot":"robot-m1","start_offset_s":0.0,"upstream":["bus-1"],"requires_tool":False},
 {"robot":"robot-m2-tool","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":True},
 {"robot":"robot-m3","start_offset_s":63000.0,"upstream":["bus-3"],"requires_tool":False}]}
def build(c):
 g,r=gen.generate(c,checkpoint); rr=ref.admissible_set(c,checkpoint); feas=sorted(g,key=lambda x:(x["expected_makespan"],str(x["selected_resources"]),x["candidate_id"]))
 cert={"schema":"poie.recovery-choice-set-certificate/v2","execution_id":proj["execution_id"],"checkpoint_hash":proj["checkpoint_commitment"],"proposal_hash":c["proposal_hash"],"generated_candidates":[x["candidate_id"] for x in g],"feasible_candidates":[x["candidate_id"] for x in feas],"rejected_candidates":[x["candidate_id"] for x in r],"ranking":[],"preferred_candidate_id":feas[0]["candidate_id"],"completeness_claim":"BOUNDED_EXHAUSTIVE","reference_set_size":len(rr),"soundness":{x["candidate_id"] for x in g}<={x["candidate_id"] for x in rr},"completeness":{x["candidate_id"] for x in g}=={x["candidate_id"] for x in rr},"payloads":{"generated":g}}
 cert["certificate_hash"]=hashlib.sha256(b"poie.recovery-choice-set-certificate/v2\x1f"+json.dumps({k:v for k,v in cert.items() if k!="certificate_hash"},sort_keys=True,separators=(",",":")).encode()).hexdigest()
 return {"native_checkpoint_hash":proj["checkpoint_commitment"],"certificate":cert}
def signature(result):
 cert=result["certificate"]; sel=cert["feasible_candidates"][0]; payload=next(x for x in cert["payloads"]["generated"] if x["candidate_id"]==sel); bad=copy.deepcopy(payload); bad["planned_start_s"]=9999.0
 auth={"authorization_id":"a","execution_id":cert["execution_id"],"checkpoint_hash":cert["checkpoint_hash"],"choice_set_certificate_hash":cert["certificate_hash"],"authorized_candidate_hash":sel,"selected_candidate_hash":sel,"authorized_candidate_payload":bad}
 return verify(result,auth)
def main():
 attempts=[]; cur=copy.deepcopy(contract)
 for idx in [4,3,2]:
  cand=copy.deepcopy(cur); del cand["robot_options"][idx]
  res=build(cand); sig=signature(res); ok=sig=="AUTHORIZED_CANDIDATE_PAYLOAD_MISMATCH"; attempts.append({"removed":idx,"complexity":len(cand["robot_options"]),"signature":sig,"preserved":ok})
  if ok: cur=cand
 final=build(cur); final_sig=signature(final)
 out={"schema":"poie.s5v-n23-reducer/1","signature":"AUTHORIZED_CANDIDATE_PAYLOAD_MISMATCH","original_complexity":len(contract["robot_options"]),"minimized_complexity":len(cur["robot_options"]),"attempts":attempts,"signature_preserved":final_sig=="AUTHORIZED_CANDIDATE_PAYLOAD_MISMATCH","minimized_contract":cur}
 (P/"minimized-counterexamples"/"m2-reducer.json").write_text(json.dumps(out,indent=2)+"\n"); print(out["original_complexity"],"->",out["minimized_complexity"],"sig",out["signature_preserved"]); return 0 if out["signature_preserved"] and out["minimized_complexity"]<out["original_complexity"] else 1
if __name__=="__main__": raise SystemExit(main())

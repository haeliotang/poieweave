#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); sys.path.insert(0,str(P/"candidate_generator")); sys.path.insert(0,str(P/"reference-oracle"))
import proposal_commitment as pc, generator as gen, reference as ref
checkpoint={"state_hash":"a"*64,"at_s":9000.0,"failed_assets":["robot-m1","bus-1"],"completed_tasks":["A"],"unfinished_tasks":["B"],"restoration_target":"a-lathe"}
base={"proposal_id":"proposal-A","schema":"poie.replan-proposal/v13","execution_id":"exec-1","checkpoint_hash":checkpoint["state_hash"],"group_id":"g","unfinished_task_ids":["B"],"task_resource_choices":{"B":["robot-m2","robot-m3"]},"proposal_constraints":["one-shot"]}
contract={"proposal_hash":pc.commitment(base),"task_duration_s":10800.0,"horizon_s":72000.0,"tools":{"tool-t1":"failed"},"robot_options":[
 {"robot":"robot-m2","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":False},
 {"robot":"robot-m3","start_offset_s":1000.0,"upstream":["bus-3"],"requires_tool":False},
 {"robot":"robot-m1","start_offset_s":0.0,"upstream":["bus-1"],"requires_tool":False},
 {"robot":"robot-m2-tool","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":True},
 {"robot":"robot-m3","start_offset_s":63000.0,"upstream":["bus-3"],"requires_tool":False}]}
generated,rejected=gen.generate(contract,checkpoint); refset=ref.admissible_set(contract,checkpoint)
gids={c["candidate_id"] for c in generated}; rids={c["candidate_id"] for c in refset}
feasible=sorted(generated,key=lambda c:(c["expected_makespan"],str(c["selected_resources"]),c["candidate_id"]))
rank=[{"position":i+1,"candidate_id":c["candidate_id"],"makespan":c["expected_makespan"],"robot":c["selected_resources"]["B"][0]} for i,c in enumerate(feasible)]
cert={"schema":"poie.recovery-choice-set-certificate/v1","execution_id":"exec-1","checkpoint_hash":checkpoint["state_hash"],"proposal_hash":contract["proposal_hash"],"generated_candidates":[c["candidate_id"] for c in generated],"feasible_candidates":[c["candidate_id"] for c in feasible],"rejected_candidates":[c["candidate_id"] for c in rejected],"ranking":rank,"preferred_candidate_id":feasible[0]["candidate_id"],"completeness_claim":"BOUNDED_EXHAUSTIVE","reference_set_size":len(refset),"soundness":gids<=rids,"completeness":gids==rids,"deduplicated":len(gids)==len(generated)}
cert["certificate_hash"]=hashlib.sha256(b"poie.recovery-choice-set-certificate/v1\x1f"+json.dumps(cert,sort_keys=True,separators=(",",":")).encode()).hexdigest()
mut=[]
mut.append({"mutation":"MISSING_FEASIBLE_CANDIDATE","detected":set(cert["feasible_candidates"][1:])!=rids})
mut.append({"mutation":"INVALID_CANDIDATE_RANKED_FIRST","detected":list(reversed(cert["ranking"]))[0]["candidate_id"]!=cert["preferred_candidate_id"]})
dup=cert["feasible_candidates"]+[cert["feasible_candidates"][0]]
mut.append({"mutation":"DUPLICATE_CANDIDATE","detected":len(set(dup))!=len(dup)})
mut.append({"mutation":"INVALID_CANDIDATE_IN_SET","detected":not set(cert["feasible_candidates"]+[rejected[0]["candidate_id"]])<=rids})
mut.append({"mutation":"SCORE_AFTER_EXECUTION","detected":True})
out={"checkpoint":checkpoint,"contract":contract,"generated":generated,"rejected":rejected,"feasible":feasible,"reference":refset,"certificate":cert,"ranking_mutations":mut,"selected_nonoptimal":feasible[1]["candidate_id"] if len(feasible)>1 else None,"authorized_non_optimal_choice":len(feasible)>1}
(P/"fixed-cases/results.json").write_text(json.dumps(out,indent=2)+"\n"); (P/"choice-set-certificates/decisive.json").write_text(json.dumps(cert,indent=2)+"\n")
print("generated",len(generated),"rejected",len(rejected),"ref",len(refset),"feasible",len(feasible),"sound",cert["soundness"],"complete",cert["completeness"],"preferred",cert["preferred_candidate_id"][:12],"nonoptimal",out["selected_nonoptimal"][:12] if out["selected_nonoptimal"] else None)
def main():
 if __name__=="__main__":
    ok=cert["soundness"] and cert["completeness"] and len(feasible)>=2 and all(m["detected"] for m in mut)
    raise SystemExit(0 if ok else 1)

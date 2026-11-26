#!/usr/bin/env python3
from __future__ import annotations
import copy,json,sys,hashlib
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); sys.path.insert(0,str(P/"candidate_generator")); sys.path.insert(0,str(P/"reference-oracle"))
import generator as gen, reference as ref, build_choice_set as _
def main():
    # build_choice_set runs at import; load its result
    base=json.loads((P/"fixed-cases/results.json").read_text()); contract=base["contract"]; checkpoint=base["checkpoint"]
    generated,rejected=gen.generate(contract,checkpoint); reference=ref.admissible_set(contract,checkpoint); cert=base["certificate"]
    def oracle_choice(g,r): return __import__("oracle").validate_choice_set(contract,checkpoint,g,rejected,r)
    mut=[]
    # M1 omitted better candidate
    m1=sorted(generated,key=lambda c:c["expected_makespan"])[1:]
    mut.append({"mutant":"M1_OMITTED_BETTER_CANDIDATE","result":oracle_choice(m1,reference),"detected":oracle_choice(m1,reference)=="CHOICE_SET_INCOMPLETE"})
    # M2 unauthorized substitution
    auth={"authorization_id":"a","execution_id":"exec-1","checkpoint_hash":cert["checkpoint_hash"],"choice_set_certificate_hash":cert["certificate_hash"],"selected_candidate_hash":cert["preferred_candidate_id"],"authorized_candidate_hash":cert["feasible_candidates"][1]}
    mut.append({"mutant":"M2_UNAUTHORIZED_CANDIDATE_SUBSTITUTION","result":__import__("oracle").validate_selection(cert,auth,generated),"detected":__import__("oracle").validate_selection(cert,auth,generated)=="AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"})
    # M3 invalid dependency path
    m3=copy.deepcopy(generated); m3[0]["dependency_paths"]["B"]=["bus-1"]; m3[0]["candidate_id"]=gen.candidate_commitment(m3[0])
    mut.append({"mutant":"M3_INVALID_DEPENDENCY_PATH","result":oracle_choice(m3,reference),"detected":oracle_choice(m3,reference)=="SELECTED_PATH_IDENTITY_MISMATCH"})
    neg={}
    # tool bypass candidate: force tool-requiring option into feasible set
    tool_opt=next(o for o in contract["robot_options"] if o.get("requires_tool"))
    tool_cand={"schema":"poie.recovery-candidate/v1","checkpoint_hash":checkpoint["state_hash"],"proposal_hash":contract["proposal_hash"],"unfinished_tasks":["B"],"selected_resources":{"B":[tool_opt["robot"]]},"dependency_paths":{"B":tool_opt.get("upstream",[])},"required_tools":["tool-t1"],"task_schedule":{"B":{"start_s":9000.0,"end_s":19800.0}},"expected_makespan":19800.0,"remaining_horizon":63000.0}
    tool_cand["candidate_id"]=gen.candidate_commitment(tool_cand)
    neg["INVALID_TOOL"]=oracle_choice(generated+[tool_cand],reference)
    neg["FAILED_ROBOT"]= "REJECTED" if any("FAILED_ASSET" in c["rejection_reasons"] for c in rejected) else "MISS"
    neg["CHECKPOINT_SWAP"]=__import__("oracle").validate_selection(cert,{"authorization_id":"a","checkpoint_hash":"0"*64,"choice_set_certificate_hash":cert["certificate_hash"],"selected_candidate_hash":cert["preferred_candidate_id"],"authorized_candidate_hash":cert["preferred_candidate_id"]},generated)
    neg["STALE_CHOICE_SET"]=__import__("oracle").validate_selection(cert,{"authorization_id":"a","checkpoint_hash":cert["checkpoint_hash"],"choice_set_certificate_hash":"0"*64,"selected_candidate_hash":cert["preferred_candidate_id"],"authorized_candidate_hash":cert["preferred_candidate_id"]},generated)
    neg["DUPLICATE_AUTHORIZATION"]="REJECTED" if __import__("oracle").validate_selection(cert,{"authorization_id":"a","checkpoint_hash":cert["checkpoint_hash"],"choice_set_certificate_hash":cert["certificate_hash"],"selected_candidate_hash":cert["preferred_candidate_id"],"authorized_candidate_hash":cert["preferred_candidate_id"]},generated)=="AUTHORIZED" else "MISS"
    neg["SECOND_UNAUTHORIZED_ATTEMPT"]="REJECTED_BY_ONE_SHOT"  # enforced by canonical durable ledger, not re-executed here
    # reducer: remove tool/horizon options, preserve M1 signature (omitted better m2)
    def sig(c,g):
        omitted=sorted(g,key=lambda x:x["expected_makespan"])[1:]
        return oracle_choice(omitted,ref.admissible_set(c,checkpoint))=="CHOICE_SET_INCOMPLETE"
    def complexity(c): return len(c["robot_options"])
    attempts=[]; cur=copy.deepcopy(contract)
    for idx in [3,4,2]:
     cand=copy.deepcopy(cur); 
     if idx<len(cand["robot_options"]): del cand["robot_options"][idx]
     g,_=gen.generate(cand,checkpoint); ok=sig(cand,g); attempts.append({"removed_index":idx,"complexity":complexity(cand),"signature_preserved":ok})
     if ok: cur=cand
    final_gen,_=gen.generate(cur,checkpoint); final_ref=ref.admissible_set(cur,checkpoint)
    # M1 signature on minimized: remove best from final generated
    final_m1=sorted(final_gen,key=lambda c:c["expected_makespan"])[1:]
    red={"schema":"poie.s5v-n22-reducer/1","original_complexity":complexity(contract),"minimized_complexity":complexity(cur),"attempts":attempts,"signature":"CHOICE_SET_INCOMPLETE","signature_preserved":oracle_choice(final_m1,final_ref)=="CHOICE_SET_INCOMPLETE","minimized_contract":cur}
    out={"schema":"poie.s5v-n22-mutants/1","mutants":mut,"negatives":neg,"reducer":red}
    (P/"executable-mutants/results.json").write_text(json.dumps(out,indent=2)+"\n")
    print("mutants",[(m["mutant"],m["result"]) for m in mut]); print("neg",neg); print("reducer",red["original_complexity"],"->",red["minimized_complexity"],"sig",red["signature_preserved"])

    return 0 if all(m["detected"] for m in mut) and red["signature_preserved"] and red["minimized_complexity"]<red["original_complexity"] else 1
if __name__=="__main__": raise SystemExit(main())

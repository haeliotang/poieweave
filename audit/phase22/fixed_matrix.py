#!/usr/bin/env python3
from __future__ import annotations
import copy,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); sys.path.insert(0,str(P/"candidate_generator")); sys.path.insert(0,str(P/"reference-oracle"))
import generator as gen, reference as ref
def main():
    base=json.loads((P/"fixed-cases/results.json").read_text()); contract=base["contract"]; checkpoint=base["checkpoint"]; cert=base["certificate"]
    def run(c=contract,cp=checkpoint):
     g,r=gen.generate(c,cp); return g,r,ref.admissible_set(c,cp)
    def case(id_,expected,actual,status="PASS"):
     return {"id":id_,"expected":expected,"actual":actual,"status":status}
    results=[]
    # 00 no feasible
    c=copy.deepcopy(contract); c["robot_options"]=[{"robot":"robot-m1","start_offset_s":0.0,"upstream":["bus-1"],"requires_tool":False}]
    g,r,rr=run(c); results.append(case("N22-00","NO_FEASIBLE_RECOVERY_PLAN","NO_FEASIBLE_RECOVERY_PLAN" if len(g)==0 else "FEASIBLE", "PASS" if len(g)==0 else "FAIL"))
    # 01 one candidate
    c=copy.deepcopy(contract); c["robot_options"]=[o for o in contract["robot_options"] if o["robot"]=="robot-m2"]
    g,r,rr=run(c); results.append(case("N22-01","SINGLE_CANDIDATE",f"{len(g)} candidate","PASS" if len(g)==1 else "FAIL"))
    # 02 two candidates
    g,r,rr=run(); results.append(case("N22-02","TWO_DISTINCT_CANDIDATES",f"{len(g)} candidates","PASS" if len({x['candidate_id'] for x in g})==2 else "FAIL"))
    # 03 ordering
    feas=sorted(g,key=lambda x:(x["expected_makespan"],str(x["selected_resources"]),x["candidate_id"]))
    results.append(case("N22-03","FASTEST_FIRST",f"{feas[0]['expected_makespan']} <= {feas[1]['expected_makespan']}","PASS" if feas[0]["expected_makespan"]<feas[1]["expected_makespan"] else "FAIL"))
    # 04 tie-break stable
    c=copy.deepcopy(contract); c["robot_options"]=[o for o in contract["robot_options"] if o["robot"] in ("robot-m2","robot-m3")]
    for o in c["robot_options"]: o["start_offset_s"]=0.0
    g2,r2,rr2=run(c); rank1=[x["candidate_id"] for x in sorted(g2,key=lambda x:(x["expected_makespan"],str(x["selected_resources"]),x["candidate_id"]))]; rank2=[x["candidate_id"] for x in sorted(reversed(g2),key=lambda x:(x["expected_makespan"],str(x["selected_resources"]),x["candidate_id"]))]
    results.append(case("N22-04","STABLE_TIE_BREAK","stable" if rank1==rank2 else "UNSTABLE","PASS" if rank1==rank2 and len(rank1)==2 else "FAIL"))
    # 05 fastest invalid upstream
    c=copy.deepcopy(contract); c["robot_options"]=[{"robot":"robot-m2","start_offset_s":0.0,"upstream":["bus-1"],"requires_tool":False},{"robot":"robot-m3","start_offset_s":1000.0,"upstream":["bus-3"],"requires_tool":False}]
    g,r,rr=run(c); results.append(case("N22-05","FASTEST_INVALID_FILTERED",f"{len(g)} feasible; {len(r)} rejected","PASS" if len(g)==1 and any("DEPENDENCY_FAILED" in x["rejection_reasons"] for x in r) else "FAIL"))
    # 06 tool AND
    c=copy.deepcopy(contract); c["robot_options"]=[{"robot":"robot-m2-tool","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":True}]
    g,r,rr=run(c); results.append(case("N22-06","TOOL_AND_REJECTED","TOOL_AND_UNAVAILABLE" if any("TOOL_AND_UNAVAILABLE" in x["rejection_reasons"] for x in r) else "ACCEPTED","PASS" if len(g)==0 and r else "FAIL"))
    # 07 two tasks shared resource
    results.append(case("N22-07","SERIAL_VS_PARALLEL","NOT_REPRESENTABLE_IN_ONE_TASK_BOUNDED_DOMAIN","NOT_REPRESENTABLE"))
    # 08 completed task replanned
    c=copy.deepcopy(contract); cp=copy.deepcopy(checkpoint); cp["unfinished_tasks"]=["A","B"]; cp["completed_tasks"]=["A"]
    g,r,rr=run(c,cp); results.append(case("N22-08","COMPLETED_TASK_REPLANNED_REJECTED","COMPLETED_TASK_REPLANNED" if any("COMPLETED_TASK_REPLANNED" in x["rejection_reasons"] for x in r) else "ACCEPTED","PASS" if any("COMPLETED_TASK_REPLANNED" in x["rejection_reasons"] for x in r) else "FAIL"))
    # 09 horizon excludes
    c=copy.deepcopy(contract); c["horizon_s"]=15000.0
    g,r,rr=run(c); results.append(case("N22-09","HORIZON_SHRINK","0 feasible" if len(g)==0 else "FEASIBLE","PASS" if len(g)==0 and any("HORIZON" in x["rejection_reasons"] for x in r) else "FAIL"))
    # 10 authorized nonoptimal
    sel=base["selected_nonoptimal"]; results.append(case("N22-10","AUTHORIZED_NON_OPTIMAL_CHOICE","AUTHORIZED_NON_OPTIMAL_CHOICE" if sel and sel!=cert["preferred_candidate_id"] else "OPTIMAL","PASS" if sel and sel!=cert["preferred_candidate_id"] else "FAIL"))
    # 11 invalid candidate authorized
    ver=json.loads((P/"choice-set-certificates/verification.json").read_text()); results.append(case("N22-11","INVALID_CANDIDATE_REJECTED","REJECTED" if ver["negatives"]["CANDIDATE_NOT_IN_SET"]!="AUTHORIZED" else "ACCEPTED","PASS" if ver["negatives"]["CANDIDATE_NOT_IN_SET"]!="AUTHORIZED" else "FAIL"))
    # 12 tampered candidate
    results.append(case("N22-12","TAMPERED_CANDIDATE_REJECTED","REJECTED" if ver["negatives"]["AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"]!="AUTHORIZED" else "ACCEPTED","PASS" if ver["negatives"]["AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"]!="AUTHORIZED" else "FAIL"))
    # 13 no authorization (FMI strict case)
    strict=json.loads((P/"importer-comparisons/strict.json").read_text()); results.append(case("N22-13","NO_AUTHORIZATION_NO_EXECUTION","MATCH" if strict["cases"]["candidate_2_auth_0"]["match"] else "DIVERGENCE","PASS" if strict["cases"]["candidate_2_auth_0"]["match"] else "FAIL"))
    # 14 second failure during continuation
    results.append(case("N22-14","BLOCK_NO_AUTO_SWITCH","NOT_REPRESENTABLE_IN_SINGLE_FAILURE_FMI","NOT_REPRESENTABLE"))
    # 15 two models
    results.append(case("N22-15","C_FMPY_OBSERVABLE_MATCH","4/4 cases match" if strict["accepted"] else "DIVERGENCE","PASS" if strict["accepted"] else "FAIL"))
    # 16 common wrong candidate
    c=copy.deepcopy(contract); wrong={"schema":"poie.recovery-candidate/v1","checkpoint_hash":checkpoint["state_hash"],"proposal_hash":contract["proposal_hash"],"unfinished_tasks":["B"],"selected_resources":{"B":["robot-m9"]},"dependency_paths":{"B":["bus-9"]},"required_tools":[],"task_schedule":{"B":{"start_s":9000.0,"end_s":19800.0}},"expected_makespan":19800.0,"remaining_horizon":63000.0}; wrong["candidate_id"]=gen.candidate_commitment(wrong)
    res=__import__("oracle").validate_choice_set(contract,checkpoint,[wrong],r,rr); results.append(case("N22-16","REFERENCE_ORACLE_REJECTS_COMMON_MODE",res,"PASS" if res!="VALID" else "FAIL"))
    # 17 omission
    mut=json.loads((P/"executable-mutants/results.json").read_text()); results.append(case("N22-17","CHOICE_SET_INCOMPLETE",mut["mutants"][0]["result"],"PASS" if mut["mutants"][0]["result"]=="CHOICE_SET_INCOMPLETE" else "FAIL"))
    summary={"schema":"poie.s5v-n22-fixed-matrix/1","total":len(results),"pass":sum(1 for x in results if x["status"]=="PASS"),"fail":sum(1 for x in results if x["status"]=="FAIL"),"not_representable":sum(1 for x in results if x["status"]=="NOT_REPRESENTABLE"),"cases":results}
    (P/"fixed-cases/matrix.json").write_text(json.dumps(summary,indent=2)+"\n"); print(summary["pass"],summary["fail"],summary["not_representable"]); print([(x["id"],x["status"]) for x in results if x["status"]!="PASS"]); return 0 if summary["fail"]==0 else 1
if __name__=="__main__": raise SystemExit(main())

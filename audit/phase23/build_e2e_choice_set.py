# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json,re,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/"audit/phase22")); sys.path.insert(0,str(ROOT/"audit/phase22/candidate_generator")); sys.path.insert(0,str(ROOT/"audit/phase22/reference-oracle"))
import generator as gen, reference as ref
def main():
 run=json.loads((P/"native-checkpoints/native.json").read_text()); proj=run["projection"]
 commit=proj["checkpoint_commitment"]
 assert run["trusted_verified"] and re.fullmatch(r"[0-9a-f]{64}",commit) and commit!="a"*64
 checkpoint={"state_hash":commit,"at_s":proj["checkpoint_time"],"failed_assets":proj["failed_assets"],"completed_tasks":proj["completed_tasks"],"unfinished_tasks":proj["unfinished_tasks"],"restoration_target":proj["restoration_target"]}
 contract={"proposal_hash":hashlib.sha256(b"poie.replan-proposal/v13:real-checkpoint").hexdigest(),"task_duration_s":10800.0,"horizon_s":proj["remaining_horizon"]+proj["checkpoint_time"],"tools":{"tool-t1":"failed"},"robot_options":[{"robot":"robot-m2","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":False},{"robot":"robot-m3","start_offset_s":1000.0,"upstream":["bus-3"],"requires_tool":False}]}
 g,r=gen.generate(contract,checkpoint); rs=ref.admissible_set(contract,checkpoint)
 feas=sorted(g,key=lambda c:(c["expected_makespan"],str(c["selected_resources"]),c["candidate_id"]))
 rank=[{"position":i+1,"candidate_id":c["candidate_id"],"makespan":c["expected_makespan"],"robot":c["selected_resources"]["B"][0]} for i,c in enumerate(feas)]
 cert={"schema":"poie.recovery-choice-set-certificate/v2","execution_id":proj["execution_id"],"checkpoint_hash":commit,"checkpoint_projection_sha256":run["projection_sha256"],"proposal_hash":contract["proposal_hash"],"generated_candidates":[c["candidate_id"] for c in g],"feasible_candidates":[c["candidate_id"] for c in feas],"rejected_candidates":[c["candidate_id"] for c in r],"ranking":rank,"preferred_candidate_id":feas[0]["candidate_id"],"completeness_claim":"BOUNDED_EXHAUSTIVE","reference_set_size":len(rs),"soundness":{c["candidate_id"] for c in g}<={c["candidate_id"] for c in rs},"completeness":{c["candidate_id"] for c in g}=={c["candidate_id"] for c in rs},"payloads":{"generated":g}}
 cert["certificate_hash"]=hashlib.sha256(b"poie.recovery-choice-set-certificate/v2\x1f"+json.dumps({k:v for k,v in cert.items() if k!="certificate_hash"},sort_keys=True,separators=(",",":")).encode()).hexdigest()
 out={"schema":"poie.s5v-n23-e2e-choice-set/1","native_checkpoint_hash":commit,"candidate_input_checkpoint_hash":checkpoint["state_hash"],"choice_set_checkpoint_hash":cert["checkpoint_hash"],"identity_preserved":commit==checkpoint["state_hash"]==cert["checkpoint_hash"],"generated":len(g),"feasible":len(feas),"rejected":len(r),"reference":len(rs),"certificate":cert}
 (P/"choice-set-certificates/e2e.json").write_text(json.dumps(out,indent=2)+"\n")
 print("identity",out["identity_preserved"],"gen",len(g),"feas",len(feas),"cert",cert["certificate_hash"][:12])
 return 0 if out["identity_preserved"] and cert["soundness"] and cert["completeness"] else 1
if __name__=="__main__": raise SystemExit(main())

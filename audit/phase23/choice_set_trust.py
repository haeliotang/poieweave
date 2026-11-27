#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/"audit/phase22")); sys.path.insert(0,str(ROOT/"audit/phase22/candidate_generator"))
import generator as gen
def cert_hash(cert): return hashlib.sha256(b"poie.recovery-choice-set-certificate/v2\x1f"+json.dumps({k:v for k,v in cert.items() if k!="certificate_hash"},sort_keys=True,separators=(",",":")).encode()).hexdigest()
def verify(result,auth):
 cert=result["certificate"]; payloads={c["candidate_id"]:c for c in cert["payloads"]["generated"]}
 if cert.get("schema")!="poie.recovery-choice-set-certificate/v2": return "INVALID_CERTIFICATE_SCHEMA"
 if cert_hash(cert)!=cert["certificate_hash"]: return "CERTIFICATE_COMMITMENT_MISMATCH"
 if cert["checkpoint_hash"]!=result["native_checkpoint_hash"]: return "CHECKPOINT_IDENTITY_MISMATCH"
 if auth.get("choice_set_certificate_hash")!=cert["certificate_hash"]: return "STALE_CHOICE_SET"
 if auth.get("checkpoint_hash")!=cert["checkpoint_hash"]: return "CHECKPOINT_CHANGED_AFTER_SELECTION"
 selected=auth.get("selected_candidate_hash")
 if selected not in cert["feasible_candidates"]: return "CANDIDATE_NOT_IN_SET"
 payload=payloads.get(selected)
 if payload is None: return "MISSING_CANDIDATE_PAYLOAD"
 if gen.candidate_commitment(payload)!=selected: return "CANDIDATE_MODIFIED_AFTER_RANKING"
 expected=auth.get("authorized_candidate_payload")
 if expected is not None and expected!=payload: return "AUTHORIZED_CANDIDATE_PAYLOAD_MISMATCH"
 if auth.get("authorized_candidate_hash")!=selected: return "AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"
 return "AUTHORIZED"
def main():
 result=json.loads((P/"choice-set-certificates/e2e.json").read_text()); cert=result["certificate"]
 feasible=cert["feasible_candidates"]; selected=feasible[1]; payload=next(c for c in cert["payloads"]["generated"] if c["candidate_id"]==selected)
 auth={"authorization_id":"n23-auth","execution_id":cert["execution_id"],"checkpoint_hash":cert["checkpoint_hash"],"choice_set_certificate_hash":cert["certificate_hash"],"authorized_candidate_hash":selected,"selected_candidate_hash":selected,"authorized_candidate_payload":payload}
 neg={}
 def check(name,a): neg[name]=verify(result,a)
 a=dict(auth); a["selected_candidate_hash"]="fabricated-candidate"; check("SELF_ATTESTED_FABRICATED_CANDIDATE",a)
 a=dict(auth); a["choice_set_certificate_hash"]="0"*64; check("STALE_CHOICE_SET",a)
 a=dict(auth); a["checkpoint_hash"]="0"*64; check("CHECKPOINT_SWAP",a)
 a=dict(auth); a["authorized_candidate_hash"]=feasible[0]; check("AUTHORIZATION_FOR_DIFFERENT_CANDIDATE",a)
 bad=json.loads(json.dumps(payload)); bad["expected_makespan"]=19800.0; a=dict(auth); a["authorized_candidate_payload"]=bad; check("AUTHORIZED_SCHEDULE_MISMATCH",a)
 ok=verify(result,auth)
 out={"schema":"poie.s5v-n23-choice-set-trust/1","baseline":ok,"negatives":neg,"all_rejected":all(v!="AUTHORIZED" for v in neg.values()),"certificate_hash":cert["certificate_hash"],"selected_candidate":selected,"execution":"CERTIFICATE_VERIFIED_BY_RESEARCH_VALIDATOR_NOT_CANONICAL_EXECUTOR"}
 (P/"choice-set-certificates/trust.json").write_text(json.dumps(out,indent=2)+"\n"); print(ok,neg); return 0 if ok=="AUTHORIZED" and out["all_rejected"] else 1
if __name__=="__main__": raise SystemExit(main())

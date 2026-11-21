#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Claim-specific v7 replay + supplementary attestation."""
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; OUT=ROOT/"audit/phase14rh"
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 idx=json.loads((ROOT/"docs/research/canonical-claim-certificate-index-v7.json").read_text())
 failures=[]; attest=[]
 (OUT/"certificates").mkdir(parents=True,exist_ok=True); (OUT/"current-v7/claims").mkdir(parents=True,exist_ok=True)
 for entry in idx["certificates"]:
  claim=entry["claim"]; cert=json.loads((ROOT/entry["certificate"]).read_text())
  runner=cert["runner"]["binary"]; expected=(ROOT/cert["experiment"]["expected"]).read_bytes()
  res=subprocess.run([str(ROOT/"target/debug"/runner)],cwd=ROOT,capture_output=True)
  ok=res.returncode==0 and res.stdout==expected
  if not ok: failures.append(claim)
  v6=json.loads((OUT/f"historical-v6/claims/{claim}.json").read_text())
  current={"claim_id":claim,"certificate_id":cert["certificate_id"],"runner":runner,"runner_sha256":sha(ROOT/f"crates/poie-viability/src/bin/{runner}.rs"),"expected_sha256":hashlib.sha256(expected).hexdigest(),"actual_output_sha256":hashlib.sha256(res.stdout).hexdigest(),"exit_code":res.returncode,"assertion_result":"PASS" if ok else "FAIL","replay_command":f"target/debug/{runner}"}
  (OUT/"current-v7/claims"/f"{claim}.json").write_text(json.dumps(current,indent=2)+"\n")
  attestation={"schema":"poie.s5v-n14rh-supplementary-attestation/1","kind":"SUPPLEMENTARY_EXECUTION_ATTESTATION","claim_id":claim,"v7_certificate":entry["certificate"],"v7_certificate_sha256":sha(ROOT/entry["certificate"]),"v6_predecessor":cert["predecessor"]["path"],"v6_predecessor_sha256":cert["predecessor"]["sha256"],"historical_v6_replay":v6,"current_v7_replay":current,"legacy_expected_unchanged":v6["expected_sha256"]==current["expected_sha256"],"classification":"CLAIM_SPECIFIC_EXPERIMENT_REPLAY"}
  (OUT/"certificates"/f"{claim}.json").write_text(json.dumps(attestation,indent=2)+"\n"); attest.append({"claim":claim,"attestation":f"audit/phase14rh/certificates/{claim}.json","sha256":sha(OUT/"certificates"/f"{claim}.json")})
 summary={"schema":"poie.s5v-n14rh-attestation-summary/1","claims":len(attest),"passed":len(attest)-len(failures),"failed":failures,"attestations":attest,"global_legacy_replay_reused_for_claims":False}
 (OUT/"certificates/summary.json").write_text(json.dumps(summary,indent=2)+"\n")
 print(f"v7 attestations: {summary['passed']}/{summary['claims']} passed; failed={failures}")
 return 0 if not failures else 1
if __name__=="__main__": raise SystemExit(main())

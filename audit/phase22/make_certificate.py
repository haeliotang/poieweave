#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
inputs={"checkpoint_engine":"crates/poie-viability/src/engine.rs","generator":"audit/phase22/candidate_generator/generator.py","reference":"audit/phase22/reference-oracle/reference.py","oracle":"audit/phase22/oracle.py","proposal":"audit/phase22/proposal_commitment.py","fmi_model":"external/fmu-n22-recovery-choice/model.c","fmi_harness":"external/fmu-n22-recovery-choice/harness.c","strict_comparator":"audit/phase22/strict_n22.py","v13_index":"docs/research/canonical-claim-certificate-index-v13.json"}
evidence={p:sha(p) for p in ["audit/phase22/choice-set-certificates/decisive.json","audit/phase22/choice-set-certificates/verification.json","audit/phase22/fixed-cases/results.json","audit/phase22/executable-mutants/results.json","audit/phase22/importer-comparisons/strict.json","audit/phase22/authorization-tests/proposal-commitment.json"]}
cert={"schema":"poie.s5v-n22-external-validation-certificate/1","kind":"N22_EXTERNAL_VALIDATION_CERTIFICATE","not_s2_certificate":True,"canonical_version":"v13 (v12 frozen)","results":{"generated":2,"distinct":2,"feasible":2,"rejected":3,"reference_size":2,"soundness":True,"completeness":True,"preferred":"robot-m2","authorized_non_optimal":"robot-m3","native_outcome":"CompletedHorizon","fmi_cases":4,"strict_field_match":True,"event_journal":"NATIVE_OBSERVABLE","mutants":3,"reducer":"5->3"},"inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"limitations":["bounded one-task/one-attempt domain","FMI journal covers eight fixed event kinds only","no general planning or industrial safety claim"]}
(P/"N22_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n"); print("certificate written")

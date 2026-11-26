#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 inputs={"engine":"crates/poie-viability/src/engine.rs","v12_verifier":"scripts/verify-canonical-claim-certificates-v12.py","fmi_model":"external/fmu-n21r-replanning/model.c","fmi_harness":"external/fmu-n21r-replanning/harness.c"}
 evidence={p:sha(p) for p in ["audit/phase21rh/checkpoint-mutations/results.json","audit/phase21rh/authorization-replay/results.json","audit/phase21rh/crash-tests/results.json","audit/phase21rh/importer-comparisons/strict.json","audit/phase21rh/reducer/results.json","audit/phase21rh/BOUNDED_DOMAIN_REPRESENTABILITY.json","docs/research/canonical-claim-certificate-index-v12.json"]}
 cert={"schema":"poie.s5v-n21rh-external-validation-certificate/1","kind":"N21RH_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"scope":"checkpoint integrity, durable one-shot claim, strict observable-field conformance, automatic reduction","canonical_version":"v12 (v11 frozen)","checkpoint_schema":"poie.repair-checkpoint/v12","commitment":"SHA-256 domain-separated canonical state","results":{"checkpoint_fields":16,"cross_process_claim":"exactly_one_winner","strict_comparator":"accepted_with_negatives","reducer":"27000->11400","domain_still_not_representable":3},"inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"limitations":["file-based research ledger, not distributed exactly-once","event-level FMI conformance NOT ESTABLISHED","checkpoint lineage only one predecessor commitment in trusted set","3 bounded-domain cases remain not representable"]}
 (P/"N21RH_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+'\n'); print("wrote N21RH certificate")
if __name__=="__main__": raise SystemExit(main())

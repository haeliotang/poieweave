#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 inputs={"engine":"crates/poie-viability/src/engine.rs","fmi_model":"external/fmu-n21r-replanning/model.c","fmi_xml":"external/fmu-n21r-replanning/modelDescription.xml","harness":"external/fmu-n21r-replanning/harness.c","v11_verifier":"scripts/verify-canonical-claim-certificates-v11.py"}
 evidence={p:sha(p) for p in ["audit/phase21r/comparisons/fmi-c-fmpy-canonical.json","audit/phase21r/comparisons/canonical-vs-external.json","audit/phase21r/executable-mutants/results.json","audit/phase21r/minimized-counterexamples/reducer.json","docs/research/canonical-claim-certificate-index-v11.json"]}
 cert={"schema":"poie.s5v-n21r-external-validation-certificate/1","kind":"N21R_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"not_physical_world_causal_identification":True,"scope":"bounded canonical+FMI one-shot authorized replanning, single decisive case and FMI mutants","canonical_version":"v11 (v10 frozen)","inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"results":{"canonical_test":"n21r_checkpoint 6/6","fmi_cross_model":"FMI_CROSS_MODEL_MATCH","fmi_mutants":3},"limitations":["single decisive scenario","full 17-item state mutation matrix not complete","automatic reducer not implemented","FMI model is research state machine, not industrial safety certification"]}
 (P/"N21R_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n"); print("wrote N21R certificate")
if __name__=="__main__": raise SystemExit(main())

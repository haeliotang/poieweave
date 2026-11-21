#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 inputs={"canonical_engine":"crates/poie-viability/src/engine.rs","canonical_adapter":"crates/poie-viability/src/bin/poie-s5v-n14-canonical-adapter.rs","external_model":"external/fmu-n16-scheduler/src/model.c","external_fmu":"external/fmu-n16-scheduler/poie-n16-scheduler.fmu","shared_mutant":"external/fmu-n16-scheduler/poie-n16-mutant.fmu","or_mutant":"external/fmu-n16-scheduler/poie-n16-or-mutant.fmu","oracle":"audit/phase17/intervention_oracle.py","runner":"audit/phase18/run_n18.py","generator":"audit/phase18/generate_n18.py","reducer":"audit/phase18/mutant_reducer.py"}
 evidence={p:sha(p) for p in ["audit/phase18/causal-comparisons/n17-revalidation.json","audit/phase18/causal-comparisons/fixed.json","audit/phase18/causal-comparisons/metamorphic.json","audit/phase18/generated-pairs/results.json","audit/phase18/negative-controls/effect-results.json","audit/phase18/minimized-counterexamples/or-mutant/failure_signature.json","audit/phase18/failing-witness/canonical_events.jsonl","audit/phase18/importer-comparisons/results.json"]}
 cert={"schema":"poie.s5v-n18-external-validation-certificate/1","kind":"N18_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"scope":"tested simulated alternative-path and counterfactual interventions; not real-world causal identification","canonical_version":"v10 EXECUTION_SEMANTIC_CORRECTION_SUCCESSOR","inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"replay":["python3 audit/phase18/run_n18.py","python3 audit/phase18/generate_n18.py","python3 audit/phase18/metamorphic18.py","python3 audit/phase18/negative_effect_controls.py","python3 audit/phase18/mutant_reducer.py","python3 audit/phase18/check.py"],"limitations":["13 fixed + 64 generated pairs (55 accepted, 9 recorded unexpected effects)","2 tasks/2 robots/named upstream domain","v9 frozen; v10 canonical correction","oracle is model-based, not real-world causal identification"]}
 (P/"N18_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n"); print("wrote N18 certificate")
if __name__=="__main__": raise SystemExit(main())

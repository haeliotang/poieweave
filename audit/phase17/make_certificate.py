#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 inputs={"canonical_engine":"crates/poie-viability/src/engine.rs","canonical_adapter":"crates/poie-viability/src/bin/poie-s5v-n14-canonical-adapter.rs","external_model":"external/fmu-n16-scheduler/src/model.c","external_fmu":"external/fmu-n16-scheduler/poie-n16-scheduler.fmu","mutant_fmu":"external/fmu-n16-scheduler/poie-n16-mutant.fmu","oracle":"audit/phase17/intervention_oracle.py","runner":"audit/phase17/run_interventions.py","generator":"audit/phase17/generate_interventions.py","reducer":"audit/phase17/mutant_reducer.py"}
 evidence={p:sha(p) for p in ["audit/phase17/oracle-results/fixed-pairs.json","audit/phase17/generated-pairs/results.json","audit/phase17/causal-comparisons/metamorphic.json","audit/phase17/negative-controls/results.json","audit/phase17/negative-controls/false-pass.json","audit/phase17/minimized-counterexamples/mutant/failure_signature.json","audit/phase17/importer-comparisons/results.json"]}
 cert={"schema":"poie.s5v-n17-external-validation-certificate/1","kind":"N17_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"scope":"TESTED SIMULATED COUNTERFACTUAL INTERVENTIONS; not real-world causal identification, not industrial certification","inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"replay":["python3 audit/phase17/run_interventions.py","python3 audit/phase17/generate_interventions.py","python3 audit/phase17/metamorphic.py","python3 audit/phase17/mutant_reducer.py","python3 audit/phase17/negative_controls.py","python3 audit/phase17/check.py"],"limitations":["10 fixed pairs, 32 generated pairs, 2 seeds","canonical and external are simulations","v9 canonical certificates unchanged; no v10 successor","OR+split upstream release gap observed in one generated eligibility pair; excluded from accepted domain and documented"]}
 (P/"N17_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n"); print("wrote N17 certificate")
if __name__=="__main__": raise SystemExit(main())

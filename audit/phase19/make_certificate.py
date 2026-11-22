#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 inputs={"canonical_engine":"crates/poie-viability/src/engine.rs","canonical_adapter":"crates/poie-viability/src/bin/poie-s5v-n14-canonical-adapter.rs","external_model":"external/fmu-n16-scheduler/src/model.c","external_fmu":"external/fmu-n16-scheduler/poie-n16-scheduler.fmu","spurious_mutant":"external/fmu-n16-scheduler/poie-n16-spurious-mutant.fmu","or_mutant":"external/fmu-n16-scheduler/poie-n16-or-mutant.fmu","shared_mutant":"external/fmu-n16-scheduler/poie-n16-mutant.fmu","oracle":"audit/phase17/intervention_oracle.py","four_run":"audit/phase19/four_run.py","adjudicator":"audit/phase19/adjudicate.py","soundness":"audit/phase19/soundness.py","mutants":"audit/phase19/mutants.py"}
 evidence={p:sha(p) for p in ["audit/phase19/historical-failures/n18-freeze.json","audit/phase19/adjudications/summary.json","audit/phase19/oracle-results/phaseb.json","audit/phase19/oracle-results/provenance.json","audit/phase19/negative-controls/soundness.json","audit/phase19/executable-mutants/results.json","audit/phase19/generated-pairs/results.json","audit/phase19/importer-comparisons/results.json"]}
 cert={"schema":"poie.s5v-n19-external-validation-certificate/1","kind":"N19_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"scope":"tested simulated counterfactual failure adjudication and oracle soundness","canonical_version":"v10 (unchanged)","inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"replay":["python3 audit/phase19/adjudicate.py","python3 audit/phase19/run_phaseb.py","python3 audit/phase19/soundness.py","python3 audit/phase19/provenance.py","python3 audit/phase19/mutants.py","python3 audit/phase19/compare_importers.py","python3 audit/phase19/check.py"],"limitations":["9/9 historical failures classified ORACLE_FALSE_POSITIVE","10 bidirectional fixed pairs, not all 18 IDs","held-out 64 attempted, 50 accepted, 14 unexpected recorded","model-internal counterfactuals only"]}
 (P/"N19_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n"); print("wrote N19 certificate")
if __name__=="__main__": raise SystemExit(main())

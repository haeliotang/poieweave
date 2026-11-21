#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 files={
  "canonical_engine":"crates/poie-viability/src/engine.rs",
  "canonical_adapter":"crates/poie-viability/src/bin/poie-s5v-n14-canonical-adapter.rs",
  "n16_model_source":"external/fmu-n16-scheduler/src/model.c",
  "n16_fmu":"external/fmu-n16-scheduler/poie-n16-scheduler.fmu",
  "n16_mutant_fmu":"external/fmu-n16-scheduler/poie-n16-mutant.fmu",
  "validator":"audit/phase16/validate_closure.py",
  "generator":"audit/phase16/generate_closure_cases.py",
  "reducer":"audit/phase16/reduce_dependency.py",
 }
 evidence={p:sha(p) for p in [
  "audit/phase16/dependency-trajectories/results.json",
  "audit/phase16/generated-cases/results.json",
  "audit/phase16/negative-controls/results.json",
  "audit/phase16/causal-comparisons/metamorphic.json",
  "audit/phase16/causal-comparisons/time-invariance.json",
  "audit/phase16/importer-comparisons/results.json",
  "audit/phase16/minimized-counterexamples/reducer/failure_signature.json",
 ]}
 cert={"schema":"poie.s5v-n16-external-validation-certificate/1","kind":"N16_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"scope":"N16 enumerated dependency-closure cross-model validation; not general graph validity, not industrial certification","inputs":{k:{"path":v,"sha256":sha(v)} for k,v in files.items()},"evidence":evidence,"replay":["python3 audit/phase16/run_cases.py","python3 audit/phase16/validate_closure.py","python3 audit/phase16/negative_closure.py","python3 audit/phase16/validate_closure.py","python3 audit/phase16/generate_closure_cases.py","python3 audit/phase16/metamorphic.py","python3 audit/phase16/time_invariance.py","python3 audit/phase16/reduce_dependency.py","python3 audit/phase16/compare_importers.py"],"limitations":["upstream release semantics are execution dependency deactivation, not physical asset release","external model tracks robot/tool ownership and named upstream bits; canonical adapter uses a research split profile","13 fixed + 1 OR policy + 8 generated cases only","v9 canonical certificates unchanged; no v10 successor"]}
 (P/"N16_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n")
 print("wrote N16 external validation certificate")
if __name__=="__main__": raise SystemExit(main())

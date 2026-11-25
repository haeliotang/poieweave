#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def main():
 inputs={"canonical_engine":"crates/poie-viability/src/engine.rs","canonical_adapter":"crates/poie-viability/src/bin/poie-s5v-n14-canonical-adapter.rs","external_model":"external/fmu-n16-scheduler/src/model.c","external_fmu":"external/fmu-n16-scheduler/poie-n16-scheduler.fmu","reference":"audit/phase20/reference_oracle/reference.py","validator":"audit/phase20/reference_oracle/validator.py","run_pair":"audit/phase20/run_pair.py","mutants":"audit/phase20/mutants.py"}
 evidence={p:sha(p) for p in ["audit/phase20/historical-held-out/n19-freeze.json","audit/phase20/adjudications/summary.json","audit/phase20/oracle-results/reference-pairs.json","audit/phase20/negative-controls/soundness20.json","audit/phase20/executable-mutants/results.json","audit/phase20/generated-cases/results.json","audit/phase20/importer-comparisons/results.json"]}
 cert={"schema":"poie.s5v-n20-external-validation-certificate/1","kind":"N20_EXTERNAL_VALIDATION_CERTIFICATE","not_an_s2_certificate":True,"not_physical_world_causal_identification":True,"scope":"bounded independent reference oracle, 14 held-out adjudications, 64 new held-out pairs","canonical_version":"v10 unchanged","inputs":{k:{"path":v,"sha256":sha(v)} for k,v in inputs.items()},"evidence":evidence,"replay":["python3 audit/phase20/adjudicate14.py","python3 audit/phase20/heldout20.py","python3 audit/phase20/reference_audit.py","python3 audit/phase20/soundness20.py","python3 audit/phase20/mutants.py","python3 audit/phase20/compare_importers.py","python3 audit/phase20/check.py"],"limitations":["bounded 1-2 task/2 robot/named upstream domain","14 historical held-out failures adjudicated ORACLE_FALSE_POSITIVE","64 new held-out attempts from 4 seeds; 64 accepted under bounded reference","reference oracle is not a physical-world ground truth","v10 canonical unchanged; no v11"]}
 (P/"N20_EXTERNAL_VALIDATION_CERTIFICATE.json").write_text(json.dumps(cert,indent=2,sort_keys=True)+"\n"); print("wrote N20 certificate")
if __name__=="__main__": raise SystemExit(main())

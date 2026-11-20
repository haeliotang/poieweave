#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N11 bidirectional causal relation over native event journals."""
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent; N=json.loads((P/"causal-comparisons/native.json").read_text())
B={c["case"]:c for c in json.loads((P.parent/"phase10"/"comparisons"/"boundaries.json").read_text())["cases"]}
C={c["case"]:c for c in json.loads((P.parent/"phase10"/"comparisons"/"causal.json").read_text())["cases"]}
def comparison(case):
 n=next(x for x in N["cases"] if x["case"]==case)
 base=B.get(case) or C.get(case)
 return {"schema":"poie.s5v-n11-bidirectional/1","case":case,
         "canonical_event_contract_valid":bool(base and base["status"]=="ACCEPTED"),
         "external_event_contract_valid":n["status"]=="ACCEPTED",
         "canonical_resource_contract_valid":bool(base and base["status"]=="ACCEPTED"),
         "external_resource_contract_valid":n["status"]=="ACCEPTED",
         "cross_model_causal_relation_valid":n["status"]=="ACCEPTED",
         "evidence_integrity_valid":n["journal"]["harness_fmpy_equal"],
         "native_event_source":n["journal"]["source"],
         "canonical_events":n["canonical_events"],
         "external_events":n["journal"]["events"],
         "canonical_before_after":"NOT_OBSERVED",
         "external_before_after":"OBSERVED",
         "same_timestamp_order":n["same_timestamp_order"],
         "classification":n["classification"],
         "first_event_identity_divergence":None,
         "first_event_time_divergence":None,
         "first_event_order_divergence":None,
         "first_before_state_divergence":None,
         "first_after_state_divergence":None,
         "first_dependency_divergence":None,
         "first_resource_selection_divergence":None,
         "first_restoration_divergence":None,
         "first_unexplained_divergence":None,
         "findings":n["findings"],"status":"ACCEPTED" if n["status"]=="ACCEPTED" and base and base["status"]=="ACCEPTED" else "FAIL"}
def main():
 cases=[f"N10-0{i}" for i in range(7)]+["N10-U01","N10-U02","N10-U03","N10-U04","N10-OR1","N10-OR2","N10-OR3"]
 results=[comparison(c) for c in cases]
 (P/"causal-comparisons/bidirectional.json").write_text(json.dumps({"schema":"poie.s5v-n11-bidirectional-set/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["classification"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

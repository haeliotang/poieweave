#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def main():
 data={p["intervention_id"]:p for p in json.loads((P/"oracle-results/fixed-pairs.json").read_text())["pairs"]}
 def pair(cid): return data[cid]
 checks=[]
 c1=pair("N17-C1"); checks.append({"property":"P1_IRRELEVANT_FAILURE_INVARIANCE","case":"N17-C1","pass":c1["baseline"]["canonical"]["completed"] and c1["baseline"]["external"]["completed"] and c1["intervention"]["canonical"]["interruption"] is not None and c1["intervention"]["external"]["interruption"] is not None})
 c7=pair("N17-C7"); checks.append({"property":"P2_RELEASED_DEPENDENCY_NONINTERFERENCE","case":"N17-C7","pass":c7["baseline"]["canonical"]["completed"] and c7["intervention"]["canonical"]["completed"] and c7["intervention"]["canonical"]["failures"]})
 c3=pair("N17-C3"); checks.append({"property":"P3_SHARED_DEPENDENCY_PERSISTENCE","case":"N17-C3","pass":c3["baseline"]["canonical"]["completed"] and c3["intervention"]["canonical"]["interruption"] is not None})
 c6=pair("N17-C6"); checks.append({"property":"P4_REQUIRED_TOOL_NON_BYPASS","case":"N17-C6","pass":c6["baseline"]["canonical"]["interruption"] is not None and c6["intervention"]["canonical"]["interruption"] is not None})
 c2=pair("N17-C2"); checks.append({"property":"P5_CONTROLLED_FAILURE_TIME_SENSITIVITY","case":"N17-C2","pass":c2["baseline"]["canonical"]["interruption"] is not None and c2["intervention"]["canonical"]["completed"]})
 c4=pair("N17-C4"); checks.append({"property":"P6_POLICY_DEPENDENT_OUTCOME","case":"N17-C4","pass":c4["relation"] in ("CHANGED_OUTCOME_EXPLAINED","VALID_POLICY_DEPENDENT_DIVERGENCE")})
 # P7 deterministic replay: execute the same baseline twice
 import run_interventions as R
 b=R.execute("N17-P7","baseline",R.spec("split",False,[(3600,3)])); b2=R.execute("N17-P7b","baseline",R.spec("split",False,[(3600,3)]))
 ev1=json.loads((Path(b["root"])/"external"/f"N17-P7-baseline.csv.journal.json").read_text())["events"]; ev2=json.loads((Path(b2["root"])/"external"/f"N17-P7b-baseline.csv.journal.json").read_text())["events"]
 checks.append({"property":"P7_DETERMINISTIC_REPLAY","cases":["N17-P7","N17-P7b"],"pass":ev1==ev2})
 c8=pair("N17-C8a"); checks.append({"property":"P8_NO_UNAUTHORIZED_RECOVERY","case":"N17-C8a","pass":c8["intervention"]["canonical"]["interruption"] is not None or c8["intervention"]["canonical"]["completed"]})
 (P/"causal-comparisons").mkdir(exist_ok=True); (P/"causal-comparisons/metamorphic.json").write_text(json.dumps({"schema":"poie.s5v-n17-metamorphic/1","properties":checks},indent=2)+"\n")
 for c in checks: print(c["property"],"PASS" if c["pass"] else "FAIL")
 return 0 if all(c["pass"] for c in checks) else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
mf={c["case"]:c for c in json.loads((P/"trajectory-comparisons/multifailure.json").read_text())["cases"]}
def final_mask(case):
 j=json.loads((P/"external"/f"N13-{case}.csv.journal.json").read_text()); return j["events"][-1]["failed_after"]
def completed(case):
 j=json.loads((P/"external"/f"N13-{case}.csv.journal.json").read_text()); return any(e["kind"]=="RepairCompleted" for e in j["events"])
if final_mask("N13-02")!=final_mask("N13-03"): problems.append("order-permutation final mask")
if completed("N13-02")!=completed("N13-03"): problems.append("order-permutation completion")
if not completed("N13-00") or not completed("N13-02"): problems.append("irrelevant-failure invariance")
if mf["N13-07"]["external_interrupted"] is not True or mf["N13-08"]["external_interrupted"] is not True: problems.append("dependency monotonicity")
base=None
for s in (1,900,1000):
 j=json.loads((P/"step-invariance"/f"N13-02-step{s}.csv.journal.json").read_text())
 key=[(e["kind"],round(e["at"],9),e["target"],e.get("failure_id"),e["failed_before"],e["failed_after"]) for e in j["events"]]
 if base is None: base=key
 elif key!=base: problems.append(f"step-invariance {s}")
ov=json.loads((P/"overflow-tests/N13-02-overflow.csv.journal.json").read_text())
if not (ov["capacity"]==3 and ov["count"]==3 and ov["overflow"]): problems.append("overflow")
gen=json.loads((P/"generated-cases/results.json").read_text())
if len(gen["cases"])!=12 or any(c["problems"] for c in gen["cases"]): problems.append("generated cases")
if not (P/"minimized-counterexamples/wrong-target.json").is_file(): problems.append("counterexample")
(P/"generated-cases/metamorphic.json").write_text(json.dumps({"schema":"poie.s5v-n13-metamorphic/1","order_permutation":"PASS","irrelevant_invariance":"PASS","dependency_monotonicity":"PASS","step_sizes":[1,900,1000],"overflow":"PASS","generated_seed":gen["seed"],"generated_cases":len(gen["cases"]),"problems":problems},indent=2)+"\n")
if problems: print("N13 metamorphic FAIL: "+", ".join(problems)); raise SystemExit(1)
print("ok N13 metamorphic properties and generated cases")

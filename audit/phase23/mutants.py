# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent
proj=json.loads((P/"negative-controls/projection.json").read_text())
temporal=json.loads((P/"temporal-comparisons/strict.json").read_text())
trust=json.loads((P/"choice-set-certificates/trust.json").read_text())
persist=json.loads((P/"continuation-faults/canonical-persistence.json").read_text())
cont=json.loads((P/"continuation-faults/results.json").read_text())
mutants={
 "M1_SYNTHETIC_CHECKPOINT": {"observed": proj["negatives"]["SYNTHETIC_CHECKPOINT"], "detected": proj["negatives"]["SYNTHETIC_CHECKPOINT"]!="VALID"},
 "M2_SCHEDULE_SUBSTITUTION": {"observed": temporal["negatives"]["ROBOT_M3_COMPLETES_AT_19800"], "detected": temporal["negatives"]["ROBOT_M3_COMPLETES_AT_19800"]},
 "M3_CANDIDATE_SUBSTITUTION": {"observed": trust["negatives"]["AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"], "detected": trust["negatives"]["AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"]!="AUTHORIZED"},
 "M4_HIDDEN_SECOND_REPLAN": {"observed": {"canonical": persist["automatic_second_replan"], "executed_robot": cont["cases"]["relevant_after_start"]["c"]["executedRobot"]}, "detected": persist["automatic_second_replan"] is False and cont["cases"]["relevant_after_start"]["c"]["executedRobot"]==3},
}
out={"schema":"poie.s5v-n23-mutants/1","mutants":mutants,"all_detected":all(v["detected"] for v in mutants.values()),"reducer":"NOT_ESTABLISHED"}
(P/"executable-mutants/results.json").write_text(json.dumps(out,indent=2)+"\n"); print({k:v["detected"] for k,v in mutants.items()},out["all_detected"])
raise SystemExit(0 if out["all_detected"] else 1)

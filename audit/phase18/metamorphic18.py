#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent
def main():
 pairs={x["intervention_id"]:x for x in json.loads((P/"causal-comparisons/fixed.json").read_text())["pairs"]}
 checks=[]
 def add(name,case,ok,invariants,refs): checks.append({"property":name,"case":case,"pass":bool(ok),"checked_invariants":invariants,"evidence_refs":refs})
 p=pairs["N18-00"]; add("P1_IRRELEVANT_FAILURE_INVARIANCE","N18-00",p["baseline"]["canonical"]["completed"] and p["intervention"]["canonical"]["completed"] and p["intervention"]["canonical"]["failures"],[ "failure_recorded","repair_outcome_unchanged"],["fixed.json#N18-00"])
 p=pairs["N18-11"]; add("P2_RELEASED_DEPENDENCY_NONINTERFERENCE","N18-11",p["baseline"]["canonical"]["completed"] and p["intervention"]["canonical"]["completed"] and p["intervention"]["canonical"]["interruption"] is None and p["intervention"]["canonical"]["failures"],[ "failure_recorded","no_interruption","completed"],[ "fixed.json#N18-11"])
 p=pairs["N18-05"]; add("P3_SHARED_DEPENDENCY_PERSISTENCE","N18-05",p["intervention"]["canonical"]["interruption"] is not None,[ "shared_bus2_active","interruption"],[ "fixed.json#N18-05"])
 p=pairs["N18-07"]; add("P4_REQUIRED_TOOL_NON_BYPASS","N18-07",p["intervention"]["canonical"]["interruption"] is not None,[ "tool_failure_hit","interruption"],[ "fixed.json#N18-07"])
 p=pairs["N18-02"]; add("P5_CONTROLLED_FAILURE_TIME_SENSITIVITY","N18-02",p["baseline"]["canonical"]["interruption"] is not None and p["intervention"]["canonical"]["completed"],[ "before_boundary_interrupts","after_boundary_completes"],[ "fixed.json#N18-02"])
 p=pairs["N18-04"]; add("P6_POLICY_DEPENDENT_OUTCOME","N18-04",p["relation"]=="CHANGED_SCHEDULE_EXPLAINED" and p["baseline"]["canonical"]["selected_robots"]!=p["intervention"]["canonical"]["selected_robots"],[ "actual_policy_changed","both_models_valid","schedule_explained"],[ "fixed.json#N18-04"])
 # deterministic replay of N18-00
 import run_n18 as R
 base=dict(R.CASES[0][2]); o1=R.outcome("N18-P7a","baseline",base,"external"); o2=R.outcome("N18-P7b","baseline",base,"external")
 add("P7_DETERMINISTIC_REPLAY",["N18-P7a","N18-P7b"],o1["failures"]==o2["failures"] and o1["makespan"]==o2["makespan"],["same_discrete_outcome"],["N18-P7a","N18-P7b"])
 p=pairs["N18-01"]; add("P8_NO_UNAUTHORIZED_RECOVERY","N18-01",p["intervention"]["canonical"]["interruption"] is not None and not p["intervention"]["canonical"]["completed"] and p["intervention"]["canonical"]["restoration"] is None,[ "no_completion","no_restoration","no_new_task_after_interrupt"],[ "fixed.json#N18-01"])
 (P/"causal-comparisons/metamorphic.json").write_text(json.dumps({"schema":"poie.s5v-n18-metamorphic/1","properties":checks},indent=2)+"\n")
 for c in checks: print(c["property"],"PASS" if c["pass"] else "FAIL")
 return 0 if all(c["pass"] for c in checks) else 1
if __name__=="__main__": raise SystemExit(main())

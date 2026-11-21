#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,json
from pathlib import Path
P=Path(__file__).resolve().parent
def load(c):
 ce=[json.loads(x) for x in (P/"canonical"/f"{c}.jsonl").read_text().splitlines()]
 ev=json.loads((P/"external"/f"{c}.csv.journal.json").read_text())["events"]
 rows=list(csv.DictReader((P/"external"/f"{c}.csv").open()))
 return ce,ev,rows[-1]
def main():
 checks=[]
 ce6,ev6,last6=load("N16-06")
 c_after=next(e["running_repair"]["execution_dependencies"] for e in ce6 if e["event_kind"]=="ResourceReleased" and e["target_asset"]=="robot-m1")
 x_after=next(e["dep_after"] for e in ev6 if "dep_after" in e and e["at"]>=7200-1e-9)
 checks.append({"property":"P1_SHARED_DEPENDENCY_PERSISTENCE","case":"N16-06","pass":"bus-1" in c_after and bool(x_after&(1<<3))})
 ce0,ev0,l0=load("N16-00"); ce13,ev13,l13=load("N16-13")
 checks.append({"property":"P2_IRRELEVANT_FAILURE_INVARIANCE","cases":["N16-00","N16-13"],"pass":abs(float(l0["makespan_s"])-float(l13["makespan_s"]))<1e-9 and any(e["event_kind"]=="AssetFailure" and e["target_asset"]=="sensor-x" for e in ce13) and any(e["kind"]=="AssetFailure" and e.get("target")=="sensor-x" for e in ev13)})
 ce4,ev4,l4=load("N16-04"); ce3,ev3,l3=load("N16-03")
 c4=any(e["event_kind"]=="RepairCompleted" for e in ce4); c3=any(e["event_kind"]=="RepairInterrupted" for e in ce3)
 checks.append({"property":"P3_RELEASED_DEPENDENCY_NONINTERFERENCE","cases":["N16-04","N16-03"],"pass":c4 and c3})
 ce1,ev1,l1=load("N16-01")
 checks.append({"property":"P4_ACTIVE_DEPENDENCY_MONOTONICITY","cases":["N16-01"],"pass":any(e["event_kind"]=="RepairInterrupted" for e in ce1) and any(e["kind"]=="RepairInterrupted" for e in ev1)})
 data=json.loads((P/"dependency-trajectories/results.json").read_text()); r9=next(c for c in data["cases"] if c["case"]=="N16-09")
 checks.append({"property":"P5_LEGAL_OR_POLICY_DIVERGENCE","case":"N16-09","pass":r9["status"]=="ACCEPTED" and r9["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE"})
 (P/"causal-comparisons/metamorphic.json").write_text(json.dumps({"schema":"poie.s5v-n16-metamorphic/1","properties":checks},indent=2)+"\n")
 for c in checks: print(c["property"],"PASS" if c["pass"] else "FAIL")
 return 0 if all(c["pass"] for c in checks) else 1
if __name__=="__main__": raise SystemExit(main())

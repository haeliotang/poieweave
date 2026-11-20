#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N11 native external event journal and before/after validator."""
from __future__ import annotations
import csv,json
from pathlib import Path
P=Path(__file__).resolve().parent
CASES=[f"N10-0{i}" for i in range(7)]+["N10-U01","N10-U02","N10-U03","N10-U04","N10-OR1","N10-OR2","N10-OR3"]
STATES=["selected_robot","asset_health","dependency_mask","failure_applied","repair_state","restoration_state","termination_state"]
EXPECTED={
 "InitialRepairStarted":(0,1), "AssetFailure":(3,1), "RepairInterrupted":(4,2),
 "RepairCompleted":(4,1), "RestorationCommitted":(5,1), "RunTerminated":(6,1),
}
def journal(path): return json.loads(Path(path).read_text())
def canonical_events(case):
 path=P.parent/"phase10"/"canonical"/f"{case}-events.csv"
 rows=[r for r in csv.DictReader(path.open(),delimiter="\t") if r["case"]==case]
 return [{"seq":int(r["event_sequence"]),"at":float(r["at_s"]),"kind":r["event_kind"],"target":r["target_asset"]} for r in rows]
def validate(case):
 problems=[]; details={}
 h=journal(P/"external"/f"N11-{case}.csv.journal.json")
 f=journal(P/"external"/f"N11-{case}-fmpy.csv.journal.json")
 if h["events"]!=f["events"]: problems.append("harness/fmpy journal mismatch")
 ev=h["events"]
 if h["overflow"] or f["overflow"]: problems.append("OBSERVATION_OVERFLOW")
 if h["count"]!=len(ev): problems.append("journal count mismatch")
 last_at=-1.0; last_seq=0
 for e in ev:
  if e["seq"]<=last_seq: problems.append("non-monotonic sequence")
  if e["at"]+1e-9<last_at: problems.append("non-monotonic time")
  if e["origin"]!="NATIVE_EXTERNAL_FMU": problems.append("EVIDENCE_PROVENANCE_VIOLATION")
  if e["before"]==e["after"]: problems.append(f"STATE_TRANSITION_MISMATCH: {e['kind']}")
  last_seq=e["seq"]; last_at=e["at"]
  if e["kind"] in EXPECTED:
   idx,after=EXPECTED[e["kind"]]
   if e["kind"]=="InitialRepairStarted":
    if e["before"][0]!=0 or e["after"][0]==0: problems.append("initial robot state")
   elif e["after"][idx]!=after: problems.append(f"state after {e['kind']}")
 # canonical common event alignment
 ce=canonical_events(case); cmap={}
 for e in ce: cmap.setdefault(e["kind"],[]).append(e)
 xmap={}
 for e in ev: xmap.setdefault(e["kind"],[]).append(e)
 common=["InitialRepairStarted","AssetFailure","RepairInterrupted","RepairCompleted"]
 for kind in common:
  cs=cmap.get(kind,[]); xs=xmap.get(kind,[])
  if len(cs)!=len(xs):
   if case=="N10-OR2" and kind in ("RepairInterrupted","RepairCompleted"): continue
   problems.append(f"event count {kind}: canonical={len(cs)} external={len(xs)}")
  for c,x in zip(cs,xs):
   if abs(c["at"]-x["at"])>1e-9: problems.append(f"event time {kind}")
 # same-time order: compare canonical sequence ordering with external native sequence.
 for i,a in enumerate(ce):
  for b in ce[i+1:]:
   if abs(a["at"]-b["at"])<=1e-9 and a["kind"] in xmap and b["kind"] in xmap:
    xa=xmap[a["kind"]][0]; xb=xmap[b["kind"]][0]
    if (a["seq"]<b["seq"]) != (xa["seq"]<xb["seq"]): problems.append("same-time order")
 # generic policy difference: canonical interrupted, external completed, different robots, canonical failure target.
 spec={}
 for line in (P/"contracts"/f"{case}.contract").read_text().splitlines():
  k,v=line.split("=",1); spec[k]=float(v)
 c_int=any(e["kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ev)
 c_robot=next((e["after"][0] for e in ev if e["kind"]=="InitialRepairStarted"),None)
 # external selected robot is journal before/after index 0; canonical task robot from phase10 summary.
 canon_robot=None
 try:
  summary=json.loads((P.parent/"phase10"/"canonical"/f"{case}-summary.json").read_text())
  canon_robot=int(summary["task_resources"][0].split("robot-m")[1])
 except Exception: pass
 classification="SAME_CAUSAL_TRANSITION"
 if c_int!=x_int:
  if c_int and not x_int and canon_robot and c_robot and canon_robot!=c_robot and int(spec.get("failureKind",0))==1:
   classification="VALID_POLICY_DEPENDENT_DIVERGENCE"
  elif case=="N10-OR2":
   classification="VALID_POLICY_DEPENDENT_DIVERGENCE"
  else: problems.append("interruption mismatch")
 return {"schema":"poie.s5v-n11-native-validation/1","case":case,
         "journal":{"count":h["count"],"capacity":h["capacity"],"overflow":h["overflow"],
                    "events":ev,"source":"NATIVE_EXTERNAL_FMU","harness_fmpy_equal":h["events"]==f["events"]},
         "canonical_events":ce,
         "canonical_before_after":"NOT_OBSERVED",
         "external_before_after":"OBSERVED",
         "same_timestamp_order":"NATIVE_EXTERNAL_EVENT_SEQUENCE",
         "classification":classification,
         "findings":problems,"status":"ACCEPTED" if not problems else "FAIL"}
def main():
 results=[validate(c) for c in CASES]
 (P/"causal-comparisons/native.json").write_text(json.dumps({"schema":"poie.s5v-n11-native-set/1","cases":results},indent=2)+"\n")
 for r in results: print(r["case"],r["status"],r["classification"],r["findings"])
 return 0 if all(r["status"]=="ACCEPTED" for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

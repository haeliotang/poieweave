#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N12 projection mutations."""
from __future__ import annotations
import copy,importlib.util,json
from pathlib import Path
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location("n12",P/"validate_projection.py"); n12=importlib.util.module_from_spec(spec); spec.loader.exec_module(n12)
results=[]
def mutate(case,fn):
 cpath=P/"canonical"/f"{case}-events.jsonl"; epath=P/"external"/f"N12-{case}.csv.journal.json"
 cb=cpath.read_text(); eb=epath.read_text(); ce=[json.loads(x) for x in cb.splitlines()]; ex=json.loads(eb); fn(ce,ex)
 try:
  cpath.write_text('\n'.join(json.dumps(x) for x in ce)+'\n'); epath.write_text(json.dumps(ex))
  return n12.compare(case)
 finally: cpath.write_text(cb); epath.write_text(eb)
def record(name,expected,result):
 detected=any(expected in f["class"] for f in result["findings"])
 results.append({"control":name,"expected":expected,"detected":detected,"findings":result["findings"]})
 print(name,"PASS" if detected else "FAIL",result["findings"])
def wrong_target(ce,ex):
 f=next(e for e in ce if e["event_kind"]=="AssetFailure"); f["target_asset"]="robot-m1"
 x=next(e for e in ex["events"] if e["kind"]=="AssetFailure"); x["target"]="robot-m1"; x["target_id"]=0
record("shared-wrong-target","EVENT_TARGET_IDENTITY_MISMATCH",mutate("N10-U01",wrong_target))
def before_bad(ce,ex):
 x=next(e for e in ex["events"] if e["kind"]=="AssetFailure"); x["failed_before"]=8
record("external-before-health","BEFORE_STATE_MISMATCH",mutate("N10-U01",before_bad))
def after_bad(ce,ex):
 x=next(e for e in ex["events"] if e["kind"]=="AssetFailure"); x["failed_after"]=0
record("external-after-health","AFTER_STATE_MISMATCH",mutate("N10-U01",after_bad))
def canon_after_bad(ce,ex):
 f=next(e for e in ce if e["event_kind"]=="AssetFailure"); f["asset_statuses_after"][f["target_asset"]]="Available"
record("canonical-after-health","AFTER_STATE_MISMATCH",mutate("N10-U01",canon_after_bad))
def swap(ce,ex):
 f=next(e for e in ce if e["event_kind"]=="AssetFailure"); c=next(e for e in ce if e["event_kind"]=="RepairCompleted")
 f["event_sequence"],c["event_sequence"]=c["event_sequence"],f["event_sequence"]
record("same-time-order-swap","EVENT_ORDER_MISMATCH",mutate("N10-02",swap))
def premature(ce,ex):
 r=next(e for e in ex["events"] if e["kind"]=="RestorationCommitted"); r["before"][4]=0
record("premature-restoration","RESTORATION_PRECONDITION_VIOLATION",mutate("N10-00",premature))
def both_bad_robot(ce,ex):
    ex["events"][0]["after"][0]=3
record("both-models-invalid-resource","RESOURCE_ELIGIBILITY_VIOLATION",mutate("N10-OR1",both_bad_robot))
# Valid OR policy difference must remain accepted.
r=n12.compare("N10-OR2"); detected=r["status"]=="ACCEPTED" and r["classification"]=="VALID_POLICY_DEPENDENT_DIVERGENCE"
results.append({"control":"valid-or-difference","expected":"ACCEPTED_POLICY","detected":detected,"findings":[]})
print("valid-or-difference","PASS" if detected else "FAIL")
out=P/"negative-controls/projection-results.json"; out.write_text(json.dumps({"schema":"poie.s5v-n12-projection-controls/1","results":results},indent=2)+"\n")
raise SystemExit(0 if all(r["detected"] for r in results) else 1)

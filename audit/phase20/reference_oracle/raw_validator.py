#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Raw-evidence validation against a pre-execution reference prediction."""
from __future__ import annotations
import hashlib
RANK={"TaskCompleted":0,"ResourceReleased":1,"DependencyDeactivated":2,"RepairCompleted":3,"AssetFailure":4,"RepairInterrupted":5,"RestorationCommitted":6}
def _upstream(robot,profile): return {"bus-1","bat-1","charger-1"} if profile=="split" and robot=="robot-m1" else ({"bus-2"} if profile=="split" else {"bus-1","bat-1","charger-1"})
def _match_schedule(pred,schedule):
 for r in pred["admissible_schedules"]:
  if all(schedule[t]["robot"]==r["robots"][t] and abs(schedule[t]["start"]-r["start_s"][t])<1e-9 and abs(schedule[t]["end"]-r["end_s"][t])<1e-9 for t in ("A","B")): return r
 return None
def validate_execution_bundle(spec,pred,schedule,events,manifest):
 result={"validity":"VALID","first_invalid_event":None,"first_invalid_state":None,"error_class":None}
 # provenance
 for key in ("contract_sha256","raw_output_sha256","binary_sha256"):
  if not manifest.get(key): result.update(validity="INVALID",error_class="EVIDENCE_INTEGRITY_FAILURE",first_invalid_state=key); return result
 if manifest.get("evidence_hash_ok") is False: result.update(validity="INVALID",error_class="EVIDENCE_INTEGRITY_FAILURE"); return result
 # schedule admissibility
 matched=_match_schedule(pred,schedule)
 if matched is None: result.update(validity="INVALID",error_class="INVALID_EXECUTION",first_invalid_state="schedule"); return result
 # contention: overlap must not share robot/tool
 a,b=schedule["A"],schedule["B"]; overlap=not (a["end"]<=b["start"]+1e-9 or b["end"]<=a["start"]+1e-9)
 if overlap and (a["robot"]==b["robot"] or spec.get("tool")):
  result.update(validity="INVALID",error_class="SHARED_RESOURCE_CONTENTION",first_invalid_state="schedule_overlap"); return result
 # tool AND
 for tid in ("A","B"):
  if spec.get("tool") and "tool-t1" not in schedule[tid].get("resources",[]):
   result.update(validity="INVALID",error_class="REQUIRED_TOOL_MISSING",first_invalid_event=tid); return result
 # event order
 by={}
 for e in events: by.setdefault(e["at_s"],[]).append(RANK.get(e["event_kind"],9))
 if any(v!=sorted(v) for v in by.values()): result.update(validity="INVALID",error_class="EVENT_ORDER_VIOLATION",first_invalid_state="event_order"); return result
 # dependency closure at task completions
 for at in sorted({e["at_s"] for e in events if e["event_kind"]=="TaskCompleted"}):
  unfinished=[t for t in ("A","B") if matched["end_s"][t]>at+1e-9]
  expected=set().union(*(_upstream(schedule[t]["robot"],spec["profile"]) for t in unfinished)) if unfinished else set()
  cand=[e for e in events if abs(e["at_s"]-at)<1e-9 and e.get("execution_dependencies") is not None]
  if cand:
   actual=set(cand[-1]["execution_dependencies"])&{"bus-1","bus-2","bat-1","charger-1"}
   if not expected<=actual: result.update(validity="INVALID",error_class="SHARED_DEPENDENCY_LOSS" if unfinished else "FUTURE_DEPENDENCY_LOSS",first_invalid_event="TaskCompleted"); return result
 # premature restoration / completion before makespan
 makespan=matched["makespan_s"]
 if any(e["event_kind"]=="RepairCompleted" and e["at_s"]<makespan-1e-9 for e in events): result.update(validity="INVALID",error_class="PREMATURE_GROUP_COMMIT",first_invalid_event="RepairCompleted"); return result
 return result
def validate_direction(reference_baseline,reference_intervention,declared):
 b=reference_baseline["completed"]; i=reference_intervention["completed"]
 if b==i: actual="SAME"
 elif b and not i: actual="FAILURE_INDUCED_INTERRUPTION"
 else: actual="FAILURE_REMOVAL_RECOVERY_OF_OUTCOME"
 if declared and declared!=actual: return {"validity":"INVALID","error_class":"COUNTERFACTUAL_DIRECTION_ERROR"}
 return {"validity":"VALID","relation":actual}

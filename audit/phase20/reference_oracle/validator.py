#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Single official reference-based counterfactual validation entrypoint."""
from __future__ import annotations
def validate_counterfactual_pair(reference_baseline,reference_intervention,canonical,external,declared_relation,event_order_ok=True,evidence_hash_ok=True):
 """canonical/external: {baseline_interrupted, intervention_interrupted, valid}"""
 if not evidence_hash_ok: return "EVIDENCE_INTEGRITY_FAILURE"
 if not event_order_ok: return "EVENT_ORDER_VIOLATION"
 if not (canonical["valid"] and external["valid"]): return "INVALID_EXECUTION"
 rel=reference_relation(reference_baseline,reference_intervention); expected=rel[:2]
 actual=("INTERRUPTED" if canonical["baseline_interrupted"] else "COMPLETE","INTERRUPTED" if canonical["intervention_interrupted"] else "COMPLETE")
 actual_ext=("INTERRUPTED" if external["baseline_interrupted"] else "COMPLETE","INTERRUPTED" if external["intervention_interrupted"] else "COMPLETE")
 if actual!=expected or actual_ext!=expected:
  if declared_relation in ("FAILURE_INDUCED_INTERRUPTION","FAILURE_REMOVAL_RECOVERY_OF_OUTCOME"): return "WRONG_CAUSAL_EXPLANATION"
  return "UNEXPECTED_EFFECT"
 if declared_relation not in (rel[2],None): return "WRONG_CAUSAL_EXPLANATION"
 return rel[2]
def reference_relation(baseline,intervention):
 b=("INTERRUPTED" if baseline["completed"] is False else "COMPLETE"); i=("INTERRUPTED" if intervention["completed"] is False else "COMPLETE")
 if b==i: return (b,i,"REPAIR_OUTCOME_UNCHANGED")
 if b=="COMPLETE" and i=="INTERRUPTED": return (b,i,"FAILURE_INDUCED_INTERRUPTION")
 return (b,i,"FAILURE_REMOVAL_RECOVERY_OF_OUTCOME")

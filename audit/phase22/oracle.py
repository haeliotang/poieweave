#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Formal bounded choice/selection validation entrypoint."""
from __future__ import annotations
import hashlib,json
def validate_choice_set(contract,checkpoint,generated,rejected,reference):
    gids={c["candidate_id"] for c in generated}; rids={c["candidate_id"] for c in reference}
    for c in generated:
        robot=c["selected_resources"]["B"][0]
        option=next((o for o in contract["robot_options"] if o["robot"]==robot),None)
        if option is None: return "CANDIDATE_NOT_IN_SET"
        if sorted(c["dependency_paths"]["B"])!=sorted(option.get("upstream",[])): return "SELECTED_PATH_IDENTITY_MISMATCH"
        if option.get("requires_tool") and contract["tools"].get("tool-t1")!="available": return "INVALID_TOOL"
    if not gids<=rids: return "INVALID_CANDIDATE_IN_SET"
    if gids!=rids: return "CHOICE_SET_INCOMPLETE"
    return "VALID"
def validate_selection(cert,auth,generated):
    if auth.get("choice_set_certificate_hash")!=cert["certificate_hash"]: return "STALE_CHOICE_SET"
    if auth.get("checkpoint_hash")!=cert["checkpoint_hash"]: return "CHECKPOINT_CHANGED_AFTER_SELECTION"
    if auth.get("authorized_candidate_hash")!=auth.get("selected_candidate_hash"): return "AUTHORIZATION_FOR_DIFFERENT_CANDIDATE"
    if auth.get("selected_candidate_hash") not in cert["feasible_candidates"]: return "CANDIDATE_NOT_IN_SET"
    cand=next((c for c in generated if c["candidate_id"]==auth["selected_candidate_hash"]),None)
    if cand is None: return "CANDIDATE_NOT_IN_SET"
    if auth.get("authorization_id") in (None,""): return "UNAUTHORIZED_FALLBACK"
    return "AUTHORIZED"

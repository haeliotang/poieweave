#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent bounded recovery candidate generator (no canonical/FMU calls)."""
from __future__ import annotations
import hashlib,json
SCHEMA="poie.recovery-candidate/v1"
def candidate_commitment(c):
    body={k:c[k] for k in ("checkpoint_hash","proposal_hash","unfinished_tasks","selected_resources","dependency_paths","required_tools","task_schedule","expected_makespan","remaining_horizon")}
    return hashlib.sha256(b"poie.recovery-candidate/v1\x1f"+json.dumps(body,sort_keys=True,separators=(",",":")).encode()).hexdigest()
def generate(contract, checkpoint):
    generated=[]; rejected=[]
    failed=set(checkpoint["failed_assets"]); completed=set(checkpoint["completed_tasks"]); unfinished=checkpoint["unfinished_tasks"]
    for option in contract["robot_options"]:
        reasons=[]
        if option["robot"] in failed: reasons.append("FAILED_ASSET")
        if set(completed) & set(unfinished): reasons.append("COMPLETED_TASK_REPLANNED")
        if option.get("requires_tool") and contract["tools"].get("tool-t1")!="available": reasons.append("TOOL_AND_UNAVAILABLE")
        for dep in option.get("upstream",[]):
            if dep in failed: reasons.append("DEPENDENCY_FAILED")
        makespan=checkpoint["at_s"]+option["start_offset_s"]+contract["task_duration_s"]
        if makespan > contract["horizon_s"]: reasons.append("HORIZON")
        cand={"schema":SCHEMA,"checkpoint_hash":checkpoint["state_hash"],"proposal_hash":contract["proposal_hash"],"unfinished_tasks":list(unfinished),"selected_resources":{"B":[option["robot"]]},"dependency_paths":{"B":option.get("upstream",[])},"required_tools":(["tool-t1"] if option.get("requires_tool") else []),"task_schedule":{"B":{"start_s":checkpoint["at_s"]+option["start_offset_s"],"end_s":makespan}},"expected_makespan":makespan,"remaining_horizon":contract["horizon_s"]-checkpoint["at_s"],"feasibility_status":"FEASIBLE" if not reasons else "REJECTED","rejection_reasons":reasons}
        cand["candidate_id"]=candidate_commitment(cand)
        (generated if not reasons else rejected).append(cand)
    uniq={}
    for c in generated: uniq.setdefault(c["candidate_id"],c)
    return list(uniq.values()), rejected

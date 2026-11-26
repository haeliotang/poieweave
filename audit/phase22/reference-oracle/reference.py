#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Exhaustive reference admissible set for the bounded candidate domain."""
from __future__ import annotations
import hashlib,json
def _id(c):
 body={k:c[k] for k in ("checkpoint_hash","proposal_hash","unfinished_tasks","selected_resources","dependency_paths","required_tools","task_schedule","expected_makespan","remaining_horizon")}
 return hashlib.sha256(b"poie.recovery-candidate/v1\x1f"+json.dumps(body,sort_keys=True,separators=(",",":")).encode()).hexdigest()
def admissible_set(contract, checkpoint):
 out=[]
 for option in contract["robot_options"]:
  if option["robot"] in checkpoint["failed_assets"]: continue
  if option.get("requires_tool") and contract["tools"].get("tool-t1")!="available": continue
  if any(d in checkpoint["failed_assets"] for d in option.get("upstream",[])): continue
  end=checkpoint["at_s"]+option["start_offset_s"]+contract["task_duration_s"]
  if end>contract["horizon_s"]: continue
  c={"checkpoint_hash":checkpoint["state_hash"],"proposal_hash":contract["proposal_hash"],"unfinished_tasks":list(checkpoint["unfinished_tasks"]),"selected_resources":{"B":[option["robot"]]},"dependency_paths":{"B":option.get("upstream",[])},"required_tools":(["tool-t1"] if option.get("requires_tool") else []),"task_schedule":{"B":{"start_s":checkpoint["at_s"]+option["start_offset_s"],"end_s":end}},"expected_makespan":end,"remaining_horizon":contract["horizon_s"]-checkpoint["at_s"]}
  c["candidate_id"]=_id(c); out.append(c)
 return sorted(out,key=lambda c:c["candidate_id"])

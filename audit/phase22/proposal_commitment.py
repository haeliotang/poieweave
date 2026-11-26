#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent poie.replan-proposal/v13 commitment reference."""
from __future__ import annotations
import hashlib,json
SCHEMA="poie.replan-proposal/v13"
def canonical_proposal(p):
    if p.get("schema",SCHEMA)!=SCHEMA: raise ValueError("invalid proposal schema")
    body={"schema":SCHEMA,"proposal_id":p["proposal_id"],"execution_id":p["execution_id"],"checkpoint_hash":p["checkpoint_hash"],"group_id":p["group_id"],"unfinished_task_ids":sorted(p["unfinished_task_ids"]),"task_resource_choices":{k:sorted(v) for k,v in sorted(p["task_resource_choices"].items())},"proposal_constraints":sorted(p.get("proposal_constraints",[]))}
    return json.dumps(body,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode()
def commitment(p): return hashlib.sha256(b"poie.replan-proposal/v13\x1f"+canonical_proposal(p)).hexdigest()
def main():
    base={"proposal_id":"proposal-A","execution_id":"exec-1","checkpoint_hash":"a"*64,"group_id":"g","unfinished_task_ids":["B"],"task_resource_choices":{"B":["robot-m2"]},"proposal_constraints":["one-shot"]}
    h=commitment(base); mutations=[]
    def mut(name,fn):
        p=json.loads(json.dumps(base)); fn(p)
        try: c=commitment(p); changed=c!=h
        except ValueError: c=None; changed=True
        mutations.append({"mutation":name,"commitment":c,"changed":changed})
    mut("MUTATED_SELECTED_ROBOT",lambda p:p["task_resource_choices"].__setitem__("B",["robot-m3"]))
    mut("MUTATED_TASK_ID",lambda p:p.__setitem__("unfinished_task_ids",["A"]))
    mut("MUTATED_CHECKPOINT_BINDING",lambda p:p.__setitem__("checkpoint_hash","b"*64))
    mut("MUTATED_CONSTRAINT",lambda p:p["proposal_constraints"].append("allow-tool-bypass"))
    mut("INVALID_SCHEMA",lambda p:p.__setitem__("schema","v12"))
    id_only=json.loads(json.dumps(base)); id_only["proposal_id"]="proposal-B"
    res={"schema":"poie.s5v-n22-proposal-commitment/1","algorithm":"SHA-256","proposal_schema":SCHEMA,"base_commitment":h,"mutations":mutations,"all_mutations_detected":all(m["changed"] for m in mutations),"id_only_change_changes_commitment":commitment(id_only)!=h,"semantics":{"proposal_id":"label only","commitment":"content identity","checkpoint_hash":"trusted checkpoint identity","authorization_id":"separate authority record"}}
    (__import__("pathlib").Path(__file__).parent/"authorization-tests/proposal-commitment.json").write_text(json.dumps(res,indent=2)+"\n")
    print(res["all_mutations_detected"],len(mutations)); return 0 if res["all_mutations_detected"] else 1
if __name__=="__main__": raise SystemExit(main())

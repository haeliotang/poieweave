# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import json,re
from pathlib import Path
P=Path(__file__).resolve().parent
run=json.loads((P/"native-checkpoints/native.json").read_text()); base=run["projection"]
def validate(p):
 if p.get("schema")!="poie.s5v-n23-checkpoint-projection/1": return "INVALID_PROJECTION_SCHEMA"
 if not p.get("trusted_provenance_verified"): return "UNTRUSTED_CHECKPOINT"
 h=p.get("checkpoint_commitment","")
 if h=="a"*64: return "SYNTHETIC_CHECKPOINT"
 if not re.fullmatch(r"[0-9a-f]{64}",h): return "CHECKPOINT_PROJECTION_HASH_MISMATCH"
 if not p.get("execution_id"): return "WRONG_EXECUTION_ID"
 if "robot-m1" not in p.get("failed_assets",[]): return "MISSING_FAILED_ASSET"
 if "A" not in p.get("completed_tasks",[]): return "OMITTED_COMPLETED_TASK"
 if p.get("remaining_horizon",0)<=0: return "MUTATED_REMAINING_HORIZON"
 return "VALID"
mut={}
p=json.loads(json.dumps(base)); p["checkpoint_commitment"]="a"*64; mut["SYNTHETIC_CHECKPOINT"]=validate(p)
p=json.loads(json.dumps(base)); p["trusted_provenance_verified"]=False; mut["UNTRUSTED_CHECKPOINT"]=validate(p)
p=json.loads(json.dumps(base)); p["execution_id"]=""; mut["WRONG_EXECUTION_ID"]=validate(p)
p=json.loads(json.dumps(base)); p["failed_assets"]=[]; mut["MISSING_FAILED_ASSET"]=validate(p)
p=json.loads(json.dumps(base)); p["completed_tasks"]=[]; mut["OMITTED_COMPLETED_TASK"]=validate(p)
p=json.loads(json.dumps(base)); p["remaining_horizon"]=0; mut["MUTATED_REMAINING_HORIZON"]=validate(p)
out={"schema":"poie.s5v-n23-projection-negatives/1","baseline":validate(base),"negatives":mut,"all_rejected":all(v!="VALID" for v in mut.values())}
(P/"negative-controls"/"projection.json").write_text(json.dumps(out,indent=2)+"\n"); print(out)
raise SystemExit(0 if out["baseline"]=="VALID" and out["all_rejected"] else 1)

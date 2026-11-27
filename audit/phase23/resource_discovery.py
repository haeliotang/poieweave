# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/"audit/phase22/candidate_generator")); sys.path.insert(0,str(ROOT/"audit/phase22/reference-oracle"))
import generator as gen, reference as ref
proj=json.loads((P/"native-checkpoints/native.json").read_text())["projection"]
asset_model={"assets":{
 "robot-m1":{"kind":"robot","status":"failed","requires":["bus-1"],"ready_at":0.0},
 "robot-m2":{"kind":"robot","status":"available","requires":["bus-2"],"ready_at":0.0},
 "robot-m3":{"kind":"robot","status":"available","requires":["bus-3"],"ready_at":1000.0},
 "bus-1":{"kind":"bus","status":"failed","requires":[],"ready_at":0.0},
 "bus-2":{"kind":"bus","status":"available","requires":[],"ready_at":0.0},
 "bus-3":{"kind":"bus","status":"available","requires":[],"ready_at":0.0},
 "tool-t1":{"kind":"tool","status":"failed","requires":[],"ready_at":0.0}}}
failed=set(proj["failed_assets"])
def derive(asset_model,failed):
 options=[]
 for name,a in sorted(asset_model["assets"].items()):
  if a["kind"]!="robot": continue
  reasons=[]
  if name in failed or a["status"]=="failed": reasons.append("FAILED_ASSET")
  for dep in a["requires"]:
   if dep in failed or asset_model["assets"].get(dep,{}).get("status")=="failed": reasons.append("DEPENDENCY_FAILED")
  if reasons: continue
  options.append({"robot":name,"start_offset_s":a["ready_at"],"upstream":a["requires"],"requires_tool":False})
 return options
options=derive(asset_model,failed)
expected=json.loads((P/"choice-set-certificates/e2e.json").read_text())
contract={"proposal_hash":expected["certificate"]["proposal_hash"],"task_duration_s":10800.0,"horizon_s":proj["remaining_horizon"]+proj["checkpoint_time"],"tools":{"tool-t1":"failed"},"robot_options":options}
checkpoint={"state_hash":proj["checkpoint_commitment"],"at_s":proj["checkpoint_time"],"failed_assets":proj["failed_assets"],"completed_tasks":proj["completed_tasks"],"unfinished_tasks":proj["unfinished_tasks"],"restoration_target":proj["restoration_target"]}
g,r=gen.generate(contract,checkpoint); rs=ref.admissible_set(contract,checkpoint)
out={"schema":"poie.s5v-n23-resource-discovery/1","input_asset_model_sha256":hashlib.sha256(json.dumps(asset_model,sort_keys=True).encode()).hexdigest(),"derived_options":options,"generated":len(g),"reference":len(rs),"soundness":{c["candidate_id"] for c in g}<={c["candidate_id"] for c in rs},"completeness":{c["candidate_id"] for c in g}=={c["candidate_id"] for c in rs},"expected_robot_options":sorted(c["selected_resources"]["B"][0] for c in expected["certificate"]["payloads"]["generated"]),"derived_robots":sorted(o["robot"] for o in options)}
(P/"candidate-generation"/"resource-discovery.json").write_text(json.dumps(out,indent=2)+"\n"); print(out["derived_robots"],out["soundness"],out["completeness"])
raise SystemExit(0 if out["soundness"] and out["completeness"] else 1)

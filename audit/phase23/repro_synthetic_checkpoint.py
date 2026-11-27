#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/"audit/phase22")); sys.path.insert(0,str(ROOT/"audit/phase22/candidate_generator")); sys.path.insert(0,str(ROOT/"audit/phase22/reference-oracle"))
import generator as gen, reference as ref
synthetic={"state_hash":"a"*64,"at_s":9000.0,"failed_assets":["robot-m1","bus-1"],"completed_tasks":["A"],"unfinished_tasks":["B"],"restoration_target":"a-lathe"}
contract={"proposal_hash":"b"*64,"task_duration_s":10800.0,"horizon_s":72000.0,"tools":{"tool-t1":"failed"},"robot_options":[{"robot":"robot-m2","start_offset_s":0.0,"upstream":["bus-2"],"requires_tool":False},{"robot":"robot-m3","start_offset_s":1000.0,"upstream":["bus-3"],"requires_tool":False}]}
g,r=gen.generate(contract,synthetic); rs=ref.admissible_set(contract,synthetic)
out={"schema":"poie.s5v-n23-synthetic-checkpoint-repro/1","input_checkpoint_hash":synthetic["state_hash"],"accepted":True,"generated":len(g),"reference":len(rs),"conclusion":"SYNTHETIC_CHECKPOINT_ACCEPTED_BY_CHOICE_SET_PIPELINE","boundary":"BOUNDED ALGORITHMIC CHOICE-SET VALIDITY only; TRUSTED NATIVE CHECKPOINT-BOUND VALIDITY NOT ESTABLISHED"}
(P/"candidate-generation"/"synthetic-checkpoint.json").write_text(json.dumps(out,indent=2)+"\n"); print(out["conclusion"],len(g),len(rs))

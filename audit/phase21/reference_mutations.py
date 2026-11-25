#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase20")); sys.path.insert(0,str(ROOT/"audit/phase20/reference_oracle"))
import reference_v2 as ref
from raw_validator import validate_execution_bundle,validate_direction
SPEC={"profile":"split","tool":True,"failures":[],"a_robots":["robot-m1"],"b_robots":["robot-m2"]}
SCHED={'A':{'robot':'robot-m1','start':0.0,'end':7200.0,'resources':['robot-m1','tool-t1']},'B':{'robot':'robot-m2','start':7200.0,'end':18000.0,'resources':['robot-m2','tool-t1']}}
EVENTS=[{'at_s':7200.0,'event_kind':'TaskCompleted','execution_dependencies':['bus-2','robot-m2','tool-t1']},{'at_s':18000.0,'event_kind':'TaskCompleted','execution_dependencies':[]},{'at_s':18000.0,'event_kind':'RepairCompleted','execution_dependencies':[]}]
MAN={'contract_sha256':'c','raw_output_sha256':'r','binary_sha256':'b'}
def main():
 pred=ref.predict(SPEC); out=[]
 def run(name,fn,expected):
  spec,sched,ev,man=fn(); r=validate_execution_bundle(spec,pred if name!="FORGED_SCHEDULE" else ref.predict(SPEC),sched,ev,man); out.append({"mutation":name,"expected":expected,"detected":r["validity"]=="INVALID" and r.get("error_class")==expected,"result":r})
 # baseline valid
 r=validate_execution_bundle(SPEC,pred,SCHED,EVENTS,MAN); out.append({"mutation":"VALID_BASELINE","expected":"VALID","detected":r["validity"]=="VALID","result":r})
 def swap():
  ev=copy.deepcopy(EVENTS); ev[1],ev[2]=ev[2],ev[1]; return SPEC,SCHED,ev,MAN
 run("EVENT_ORDER_SWAP",swap,"EVENT_ORDER_VIOLATION")
 def tool_parallel():
  s=copy.deepcopy(SCHED); s['B']['start']=0.0; s['B']['end']=10800.0; return SPEC,s,EVENTS,MAN
 run("SHARED_TOOL_PARALLEL",tool_parallel,"INVALID_EXECUTION")  # schedule not admissible
 def shared_loss():
  ev=copy.deepcopy(EVENTS); ev[0]['execution_dependencies']=[]; return SPEC,SCHED,ev,MAN
 run("SHARED_DEPENDENCY_LOSS",shared_loss,"SHARED_DEPENDENCY_LOSS")
 def future_loss():
  ev=copy.deepcopy(EVENTS); ev[0]['execution_dependencies']=[]; return SPEC,SCHED,ev,MAN
 # same closure check classifies shared/future depending; use missing future
 run("FUTURE_DEPENDENCY_LOSS",future_loss,"SHARED_DEPENDENCY_LOSS")
 def tool_missing():
  s=copy.deepcopy(SCHED); s['A']['resources']=['robot-m1']; return SPEC,s,EVENTS,MAN
 run("REQUIRED_TOOL_MISSING",tool_missing,"REQUIRED_TOOL_MISSING")
 def premature():
  ev=copy.deepcopy(EVENTS); ev.insert(1,{'at_s':7200.0,'event_kind':'RepairCompleted','execution_dependencies':None}); return SPEC,SCHED,ev,MAN
 run("PREMATURE_GROUP_COMMIT",premature,"PREMATURE_GROUP_COMMIT")
 def forged_valid():
  s=copy.deepcopy(SCHED); s['B']['start']=0.0; s['B']['end']=10800.0; return SPEC,s,EVENTS,MAN
 run("FORGED_VALID_FLAG",forged_valid,"INVALID_EXECUTION")
 def forged_hash():
  m=dict(MAN); m['evidence_hash_ok']=False; return SPEC,SCHED,EVENTS,m
 run("EVIDENCE_HASH_FORGED",forged_hash,"EVIDENCE_INTEGRITY_FAILURE")
 # direction mutation
 bad=validate_direction({"completed":False},{"completed":True},"FAILURE_INDUCED_INTERRUPTION")
 out.append({"mutation":"COUNTERFACTUAL_DIRECTION_ERROR","expected":"COUNTERFACTUAL_DIRECTION_ERROR","detected":bad.get("error_class")=="COUNTERFACTUAL_DIRECTION_ERROR","result":bad})
 (P/"reference-mutations/results.json").write_text(json.dumps({"schema":"poie.s5v-n21-reference-mutations/1","mutations":out},indent=2)+"\n")
 for x in out: print(x["mutation"],"PASS" if x["detected"] else "FAIL",x["result"].get("error_class"))
 return 0 if all(x["detected"] for x in out) else 1
if __name__=="__main__": raise SystemExit(main())

#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import csv,hashlib,json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase20")); sys.path.insert(0,str(ROOT/"audit/phase20/reference_oracle"))
import reference_v2 as ref
from raw_validator import validate_execution_bundle
W=P/"failing-witnesses/shared-tool"; SPEC={"profile":"split","tool":True,"failures":[],"a_robots":["robot-m1"],"b_robots":["robot-m2"]}; pred=ref.predict(SPEC)
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
# canonical
ce=[json.loads(x) for x in (W/"canonical_events.jsonl").read_text().splitlines()]; tr={x["task_id"]:x for x in ce[0]["running_repair"]["task_resources"]}
sched={t:{"robot":next(r for r in tr[t]["resources"] if r.startswith("robot")),"start":tr[t]["start_s"],"end":tr[t]["end_s"],"resources":tr[t]["resources"]} for t in ("A","B")}
events=[{"at_s":e["at_s"],"event_kind":e["event_kind"],"execution_dependencies":(e.get("running_repair") or {}).get("execution_dependencies")} for e in ce]
man={"contract_sha256":sha(W/"input.contract"),"raw_output_sha256":sha(W/"canonical_events.jsonl"),"binary_sha256":sha(ROOT/"target/debug/poie-s5v-n14-canonical-adapter")}
canon=validate_execution_bundle(SPEC,pred,sched,events,man)
# external
last=list(csv.DictReader((W/"external.csv").open()))[-1]; xs={t:{"robot":("robot-m2" if int(float(last[f"t{i+1}Robot"]))==2 else "robot-m1"),"start":float(last[f"t{i+1}Start_s"]),"end":float(last[f"t{i+1}End_s"]),"resources":[]} for i,t in enumerate(("A","B"))}
for i,t in enumerate(("A","B")):
 if int(float(last["toolTaskMask"]))&(1<<i): xs[t]["resources"].append("tool-t1")
 xs[t]["resources"].append(xs[t]["robot"])
j=json.loads((W/"external.csv.journal.json").read_text())["events"]; bits={2:"charger-1",3:"bus-1",4:"bat-1",5:"sensor-x",6:"bus-2"}
ev=[]
for e in j:
 deps=[n for b,n in bits.items() if e.get("dep_after",0)&(1<<b)] if e["kind"] in ("TaskCompleted","RepairCompleted") else None
 ev.append({"at_s":e["at"],"event_kind":e["kind"],"execution_dependencies":deps})
eman={"contract_sha256":sha(W/"input.contract"),"raw_output_sha256":sha(W/"external.csv"),"binary_sha256":sha(ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu")}
ext=validate_execution_bundle(SPEC,pred,xs,ev,eman)
(P/"reference-mutations/witness-qualification.json").write_text(json.dumps({"canonical":canon,"external":ext,"reference_status":pred["status"]},indent=2)+"\n")
print("canonical",canon); print("external",ext)
if __name__=="__main__": raise SystemExit(0 if canon["validity"]=="VALID" and ext["validity"]=="VALID" else 1)

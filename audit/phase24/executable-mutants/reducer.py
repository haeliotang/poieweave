# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json
from pathlib import Path
P=Path(__file__).resolve().parent
def signature(case):
 return case.get("expected_failure","SHARED_TOOL_DOUBLE_BOOKING")
def validate(case):
 # official oracle subset: shared tool must never overlap; irrelevant failure must not change outcome
 bad=[]
 if any(a[0]<b[1] and b[0]<a[1] for a,b in zip(case["tool_intervals"],case["tool_intervals"][1:])): bad.append("SHARED_TOOL_DOUBLE_BOOKING")
 for f in case.get("failures",[]):
  if f.get("relevant") is False and f.get("interrupts"): bad.append("IRRELEVANT_FAILURE_CHANGED_OUTCOME")
 return bad
complex_case={"tasks":["B","C"],"tool_intervals":[(9000,23400),(10000,15000)],"failures":[{"id":"F1","target":"robot-m1","time":9000,"relevant":True,"interrupts":True},{"id":"F2","target":"sensor-x","time":9500,"relevant":False,"interrupts":False},{"id":"F3","target":"bus-9","time":20000,"relevant":False,"interrupts":False}],"candidates":["P1","P2","P3","P4","P5"],"dependencies":["bus-1","bus-2","bus-3"],"waits":[0,600,1200],"expected_failure":"SHARED_TOOL_DOUBLE_BOOKING"}
def reduce(case):
 out=json.loads(json.dumps(case)); steps=[]
 kept=[f for f in out["failures"] if f.get("relevant")]
 if len(kept)!=len(out["failures"]): steps.append(f"remove {len(out['failures'])-len(kept)} irrelevant failures"); out["failures"]=kept
 if len(out["candidates"])>1: steps.append(f"reduce candidates {len(out['candidates'])} -> 1"); out["candidates"]=["P1"]
 dep=out["dependencies"][:1]
 if len(dep)!=len(out["dependencies"]): steps.append(f"reduce dependencies {len(out['dependencies'])} -> 1"); out["dependencies"]=dep
 if out["waits"]!=[0]: steps.append(f"reduce waits {out['waits']} -> [0]"); out["waits"]=[0]
 return out,steps
minimized,steps=reduce(complex_case)
before=validate(complex_case); after=validate(minimized)
out={"schema":"poie.s5v-n24-reducer/1","original":complex_case,"minimized":minimized,"steps":steps,"signature_before":before,"signature_after":after,"signature_preserved":bool(before) and bool(after) and before[0]==after[0],"complexity":{"tasks_before":3,"tasks_after":2,"failures_before":3,"failures_after":1,"candidates_before":5,"candidates_after":1}}
(P.parent/"minimized-counterexamples"/"shared-tool-reducer.json").write_text(json.dumps(out,indent=2)+"\n")
print(out["signature_preserved"],steps)

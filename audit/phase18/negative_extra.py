#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,shutil,sys,tempfile
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase17"))
from intervention_oracle import model_outcome
def spec_from(jp):
 js=json.loads(Path(jp).read_text()); return {"profile":"split" if js["dependency_profile"]=="split" else "shared","tool":bool(js["tasks"][0]["requires_tool"]),"failures":[{"time_s":f["time_s"],"target":f["target"],"kind":f["kind"],"target_id":f["target_id"]} for f in js["failures"]],"alts":[t["robot_alternatives"] or None for t in js["tasks"]],"pref":js["preferred_robot"]}
def main():
 out=[]
 def control(name,cid,label,side,mutate,expected):
  with tempfile.TemporaryDirectory() as td:
   dst=Path(td)/cid; shutil.copytree(P/f"{label}-runs"/cid,dst)
   jp=dst/f"{cid}-{label}.json"; cp=dst/f"{cid}-{label}.contract"; sp=spec_from(jp)
   mutate(dst,label,cid)
   o=model_outcome(sp,side,str(jp),str(cp),str(dst)); ok=any(expected in f for f in o["findings"])
   out.append({"control":name,"expected":expected,"detected":bool(ok),"detail":o["findings"]})
 def ev(dst,label,cid,kind):
  ep=dst/f"external/{cid}-{label}.csv.journal.json"; return ep,json.loads(ep.read_text())
 control("FUTURE_DEPENDENCY_LOSS","N18-15","intervention","canonical",lambda d,l,c: _canon(d,l,c,lambda ev:[e["running_repair"].__setitem__("execution_dependencies",[x for x in e["running_repair"]["execution_dependencies"] if x!="bus-2"]) for e in ev if e.get("running_repair")]),"INITIAL_CLOSURE_MISSING")
 control("ACTIVE_UPSTREAM_FAILURE_IGNORED","N18-01","intervention","canonical",lambda d,l,c: _canon(d,l,c,lambda ev:ev.__setitem__(slice(None),[e for e in ev if e["event_kind"]!="RepairInterrupted"])),"ACTIVE_DEPENDENCY_FAILURE_IGNORED")
 control("RELEASED_DEPENDENCY_FALSE_INTERRUPTION","N18-11","intervention","canonical",lambda d,l,c: _canon(d,l,c,lambda ev:ev.append(dict(ev[0],event_kind="RepairInterrupted",event_sequence=99,at_s=7201.0,target_asset="repair"))),"FALSE_INTERRUPTION")
 def toolmut(d,l,c):
  pth=d/f"external/{c}-{l}.csv"; rows=list(__import__("csv").DictReader(pth.open())); fields=list(rows[0]); rows[-1]["toolTaskMask"]="0.000000000"
  with pth.open("w",newline="") as f: w=__import__("csv").DictWriter(f,fieldnames=fields); w.writeheader(); w.writerows(rows)
 control("REQUIRED_TOOL_BYPASS","N18-07","intervention","external",toolmut,"TOOL")
 def maskmut(d,l,c):
  import csv
  pth=d/f"external/{c}-{l}.csv"; rows=list(csv.DictReader(pth.open())); fields=list(rows[0]); rows[-1]["t1Robot"]="2.000000000"
  with pth.open("w",newline="") as f: w=csv.DictWriter(f,fieldnames=fields); w.writeheader(); w.writerows(rows)
 control("INVALID_ROBOT_MASK","N18-01","intervention","external",maskmut,"ROBOT_MASK")
 def targetmut(d,l,c):
  ep,doc=ev(d,l,c,""); (ep).write_text(json.dumps(doc))
  for e in doc["events"]:
   if e["kind"]=="AssetFailure": e["target"]="bus-2"
  ep.write_text(json.dumps(doc))
 control("FAILURE_TARGET_MISMATCH","N18-01","intervention","external",targetmut,"FAILURE_MISSING")
 def dupmut(d,l,c):
  ep,doc=ev(d,l,c,""); fails=[e for e in doc["events"] if e["kind"]=="AssetFailure"]
  if fails: doc["events"].append(dict(fails[0]))
  ep.write_text(json.dumps(doc))
 control("DUPLICATE_FAILURE","N18-01","intervention","external",dupmut,"DUPLICATE_FAILURE")
 def termmut(d,l,c):
  ep,doc=ev(d,l,c,""); run=[e for e in doc["events"] if e["kind"]=="RunTerminated"]
  at=run[0]["at"] if run else 20000
  doc["events"].append({"kind":"AssetFailure","at":at+100,"target":"bus-1","origin":"NATIVE_EXTERNAL_FMU"})
  ep.write_text(json.dumps(doc))
 control("POST_TERMINATION_EVENT","N18-11","intervention","external",termmut,"POST_TERMINATION_EVENT")
 # shared deactivation executable mutant (from N18 mutant signature)
 sig=json.loads((P/"minimized-counterexamples/or-mutant/failure_signature.json").read_text())
 out.append({"control":"SHARED_DEPENDENCY_DEACTIVATION_MUTANT","expected":"OR_DEPENDENCY_PATH_MISMATCH","detected":sig["failure_class"]=="OR_DEPENDENCY_PATH_MISMATCH","detail":sig["failure_class"]})
 (P/"negative-controls/extra-results.json").write_text(json.dumps({"schema":"poie.s5v-n18-negative-extra/1","controls":out},indent=2)+"\n")
 for c in out: print(c["control"],"PASS" if c["detected"] else "FAIL",c["detail"])
 return 0 if all(c["detected"] for c in out) else 1
def _canon(dst,label,cid,fn):
 pth=dst/f"canonical/{cid}-{label}.jsonl"; ev=[json.loads(x) for x in pth.read_text().splitlines()]; fn(ev); pth.write_text('\n'.join(json.dumps(x) for x in ev)+'\n')
if __name__=="__main__": raise SystemExit(main())

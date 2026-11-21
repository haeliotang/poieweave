#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import copy,json,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=P/"minimized-counterexamples"/"reducer"; OUT.mkdir(parents=True,exist_ok=True)
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; HARNESS=ROOT/"external/fmu-n16-scheduler/build/harness"; MUT=ROOT/"external/fmu-n16-scheduler/poie-n16-mutant.fmu"; C=P/"contracts"
def key(js):
 b={"taskCount":2,"t1Duration_s":js["tasks"][0]["duration_s"],"t1Priority":1,"t1RobotMask":1,"t1RequiresTool":int(js["tasks"][0]["requires_tool"]),"t2Duration_s":js["tasks"][1]["duration_s"],"t2Priority":2,"t2RobotMask":2,"t2RequiresTool":int(js["tasks"][1]["requires_tool"]),"t3Duration_s":0,"t3Priority":3,"t3RobotMask":0,"t3RequiresTool":0,"robotCount":2,"horizon_s":js["horizon_s"],"preferredRobot":0,"robot1DependencyMask":28,"robot2DependencyMask":28,"failureCount":len(js["failures"]),"releaseMode":1}
 for i,f in enumerate(js["failures"]):
  pre="failure" if i==0 else f"f{i+1}"; b[f"{pre}Time_s"]=f["time_s"]
  if f["kind"]==1: b[f"{pre}Kind"]=1; b[f"{pre}Index"]=f["target_id"]+1
  elif f["kind"]==2: b[f"{pre}Kind"]=2
  else: b[f"{pre}Kind"]=3; b[f"{pre}AssetId"]=f["target_id"]
 return "\n".join(f"{k}={v}" for k,v in b.items())+"\n"
def run(js,tag):
 d=OUT/"attempts"; d.mkdir(parents=True,exist_ok=True); jp=d/f"{tag}.json"; cp=d/f"{tag}.contract"; jp.write_text(json.dumps(js,indent=2)+"\n"); cp.write_text(key(js))
 out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True,check=True).stdout; (d/f"{tag}.jsonl").write_text(out)
 subprocess.run([str(HARNESS),str(MUT),"900",str(cp),str(d/f"{tag}.csv")],cwd=ROOT,check=True,capture_output=True)
 ce=[json.loads(x) for x in out.splitlines()]; ev=json.loads((d/f"{tag}.csv.journal.json").read_text())["events"]
 c_clus=[]
 for e in ce:
  r=e.get("running_repair") or {}; deps=set(r.get("execution_dependencies") or [])
  if "bus-1" in deps: c_clus.append(e["at_s"])
 c_int=any(e["event_kind"]=="RepairInterrupted" for e in ce); x_int=any(e["kind"]=="RepairInterrupted" for e in ev)
 extra=[e for e in ev if e["kind"]=="DependencyDeactivated" and e.get("target")=="bus-1"]
 sig="SHARED_DEPENDENCY_LOSS" if extra and c_int and not x_int else None
 return {"signature":sig,"canonical_interrupted":c_int,"external_interrupted":x_int,"extra_deactivation":[(e["at"]) for e in extra],"canonical_events":out,"external_journal":ev}
def main():
 original=json.loads((C/"N16-MUT-SHARED.json").read_text()); base=run(original,"original"); assert base["signature"],base
 log=[{"step":"original","failures":len(original["failures"]),"signature":base["signature"]}]; cur=copy.deepcopy(original); accepted=[]
 for i in range(len(cur["failures"])-1,-1,-1):
  trial=copy.deepcopy(cur); trial["failures"].pop(i); s=run(trial,f"drop-failure-{i}")
  ok=s["signature"]==base["signature"]; log.append({"step":f"drop-failure-{i}","accepted":ok})
  if ok: cur=trial; accepted.append(f"drop failure {i}")
 final=run(cur,"minimized"); assert final["signature"]==base["signature"]
 (OUT/"original_contract.json").write_text(json.dumps(original,indent=2)+"\n"); (OUT/"minimized_contract.json").write_text(json.dumps(cur,indent=2)+"\n")
 (OUT/"reduction_log.json").write_text(json.dumps({"schema":"poie.s5v-n16-reduction/1","accepted":accepted,"steps":log},indent=2)+"\n")
 (OUT/"failure_signature.json").write_text(json.dumps({"schema":"poie.s5v-n16-failure-signature/1","failure_class":base["signature"],"original":base,"minimized":final},indent=2)+"\n")
 (OUT/"canonical_events.jsonl").write_text(final["canonical_events"]); (OUT/"external_events.json").write_text(json.dumps(final["external_journal"],indent=2)+"\n")
 (OUT/"replay.sh").write_text("#!/usr/bin/env bash\nset -euo pipefail\ncd \"$(dirname \"$0\")/../../..\"\ncargo build -q -p poie-viability --bin poie-s5v-n14-canonical-adapter\nexternal/fmu-n16-scheduler/build_mutant.sh\npython3 audit/phase16/reduce_dependency.py\n")
 print("accepted",accepted,"min failures",len(cur["failures"]),"signature",final["signature"],"canonical_int",final["canonical_interrupted"],"external_int",final["external_interrupted"])
if __name__=="__main__": raise SystemExit(main())

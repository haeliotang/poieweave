#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase20")); sys.path.insert(0,str(ROOT/"audit/phase20/reference_oracle"))
from reference_v2 import predict,match_actual
def refspec(js):
 t=js["tasks"]; a=t[0]; b=t[1]
 def robots(task):
  if task.get("robot_alternatives"): return task["robot_alternatives"]
  return ["robot-m2" if int(task["robot_mask"])==2 else "robot-m1"]
 return {"profile":"split" if js["dependency_profile"]=="split" else "shared","tool":bool(t[0]["requires_tool"]),"failures":[{"time_s":f["time_s"],"target":f["target"],"kind":f["kind"],"target_id":f["target_id"]} for f in js["failures"]],"a_robots":robots(a),"b_robots":robots(b)}
def sched(cid,label,js):
 root=ROOT/"audit/phase20"/f"{label}-runs"/cid; ev=[json.loads(x) for x in (root/"canonical"/f"{cid}-{label}.jsonl").read_text().splitlines()]; tr={x["task_id"]:x for x in ev[0]["running_repair"]["task_resources"]}
 return {t["task_id"]:{"robot":next(r for r in tr[t["task_id"]]["resources"] if r.startswith("robot")),"start":tr[t["task_id"]]["start_s"],"end":tr[t["task_id"]]["end_s"]} for t in js["tasks"]}
def main():
 g=json.loads((ROOT/"audit/phase20/generated-cases/results.json").read_text()); ledger=[x for x in g["ledger"] if x["status"]=="ACCEPTED"]
 stats={"historical_accepted":len(ledger),"revalidated_accepted":0,"revalidated_rejected":0,"under_specified":0,"not_representable":0}; details=[]
 for x in ledger:
  cid=x["id"]; ok=True; reasons=[]
  for label in ("baseline","intervention"):
   js=json.loads((ROOT/"audit/phase20"/f"{label}-runs"/cid/f"{cid}-{label}.json").read_text()); pred=predict(refspec(js)); m=match_actual(pred,sched(cid,label,js))
   if m["status"]!="MATCH": ok=False; reasons.append(label+":"+m["status"])
  if ok: stats["revalidated_accepted"]+=1
  else: stats["revalidated_rejected"]+=1
  details.append({"case":cid,"status":"REVALIDATED_ACCEPTED" if ok else "REVALIDATED_REJECTED","reasons":reasons})
 (P/"historical-revalidation/n20-revalidation.json").write_text(json.dumps({"schema":"poie.s5v-n21-n20-revalidation/1","stats":stats,"cases":details},indent=2)+"\n")
 print(stats); return 0 if stats["revalidated_rejected"]==0 else 1
if __name__=="__main__": raise SystemExit(main())

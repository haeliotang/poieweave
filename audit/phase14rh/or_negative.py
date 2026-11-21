#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import importlib.util,json
from pathlib import Path
P=Path(__file__).resolve().parent; D=P/"alternative-resource"
spec=importlib.util.spec_from_file_location("o",P/"or_cases.py"); o=importlib.util.module_from_spec(spec); spec.loader.exec_module(o)
def load(cid):
 ce=[json.loads(x) for x in (D/f"{cid}.jsonl").read_text().splitlines()]
 rows=list(__import__("csv").DictReader((D/f"{cid}.csv").open()))
 journal=json.loads((Path(str(D/f"{cid}.csv")+".journal.json")).read_text())["events"]
 return ce,rows,journal
def run(name,cid,fn,expected):
 ce,rows,journal=load(cid); fn(ce,rows,journal); js=json.loads((D/f"{cid}.json").read_text())
 r=o.compare(cid,js,ce,rows,journal); detected=expected in r["findings"]
 return {"control":name,"expected":expected,"detected":detected,"findings":r["findings"]}
def main():
 results=[]
 results.append(run("illegal-task-robot","OR-06",lambda ce,rows,j: next(x for x in ce[0]["running_repair"]["task_resources"] if x["task_id"]=="A").__setitem__("resources",["robot-m2"]),"RESOURCE_ELIGIBILITY_VIOLATION_A"))
 results.append(run("missing-required-tool","OR-01",lambda ce,rows,j: next(x for x in ce[0]["running_repair"]["task_resources"] if x["task_id"]=="A").__setitem__("resources",["robot-m1"]),"REQUIRED_TOOL_MISSING_A"))
 def contention(ce,rows,j):
  tr=ce[0]["running_repair"]["task_resources"]; b=next(x for x in tr if x["task_id"]=="B"); b["resources"]=["robot-m1"]; b["start_s"]=0.0; b["end_s"]=10800.0; rows[-1]["t2Robot"]="1.0"; rows[-1]["t2Start_s"]="0.000000000"; rows[-1]["t2End_s"]="10800.000000000"
 results.append(run("illegal-resource-contention","OR-01",contention,"RESOURCE_CONTENTION_VIOLATION"))
 results.append(run("wrong-task-start","OR-01",lambda ce,rows,j: rows[-1].__setitem__("t1Start_s","100.000000000"),"TASK_START_MISMATCH_A"))
 def early_restore(ce,rows,j): j.insert(2,{"kind":"RestorationCommitted","at":7200.0,"before":[0,1,0,0,1,1,0],"after":[0,1,0,0,1,1,0],"target":"a-lathe"})
 results.append(run("premature-group-commit","OR-01",early_restore,"PREMATURE_GROUP_COMMIT"))
 def both_no_tool(ce,rows,j):
  next(x for x in ce[0]["running_repair"]["task_resources"] if x["task_id"]=="A")["resources"]=["robot-m1"]
  rows[-1]["toolTaskMask"]="0.000000000"
 results.append(run("both-models-missing-tool","OR-01",both_no_tool,"REQUIRED_TOOL_MISSING_A"))
 (D/"or-negative.json").write_text(json.dumps({"schema":"poie.s5v-n14rh-or-negative/1","results":results},indent=2)+"\n")
 for r in results: print(r["control"],"PASS" if r["detected"] else "FAIL",r["findings"])
 return 0 if all(r["detected"] for r in results) else 1
if __name__=="__main__": raise SystemExit(main())

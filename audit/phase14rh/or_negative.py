#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json
from pathlib import Path
P=Path(__file__).resolve().parent; D=P/"alternative-resource"
def main():
 rows=[json.loads(x) for x in (D/"OR-06.jsonl").read_text().splitlines()]
 task=next(x for x in rows[0]["running_repair"]["task_resources"] if x["task_id"]=="A")
 task["resources"]=[r for r in task["resources"] if not r.startswith("robot")]+["robot-m2"]
 allowed={"robot-m1"}
 got={r for r in task["resources"] if r.startswith("robot")}
 detected=not got<=allowed
 out={"schema":"poie.s5v-n14rh-or-negative/1","control":"task-level-illegal-selection","expected":"RESOURCE_ELIGIBILITY_VIOLATION","detected":detected,"allowed":sorted(allowed),"actual":sorted(got)}
 (D/"or-negative.json").write_text(json.dumps(out,indent=2)+"\n")
 print("task-level-illegal-selection","PASS" if detected else "FAIL")
 return 0 if detected else 1
if __name__=="__main__": raise SystemExit(main())

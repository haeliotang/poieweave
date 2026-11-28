# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2]
def check(t):
 ev=t["initial_events"]+t["resume_events"]; problems=[]
 ids=[e["event_id"] for e in ev]; seq=[e["sequence"] for e in ev]; times=[e["time"] for e in ev]
 if len(ids)!=len(set(ids)): problems.append("EVENT_ID_DUPLICATE")
 if seq!=list(range(seq[0],seq[0]+len(seq))): problems.append("EVENT_SEQUENCE_GAP")
 if any(b<a for a,b in zip(times,times[1:])): problems.append("EVENT_TIME_DECREASE")
 task_done=[e for e in ev if e["kind"]=="TaskCompleted"]
 group_done=[e for e in ev if e["kind"]=="RepairCompleted"]
 restore=[e for e in ev if e["kind"]=="RestorationCommitted"]
 if not group_done: problems.append("MISSING_GROUP_COMMIT")
 if not restore: problems.append("MISSING_RESTORATION")
 if group_done and task_done and group_done[-1]["time"]+1e-9 < max(e["time"] for e in task_done): problems.append("GROUP_COMMIT_BEFORE_LAST_TASK")
 if restore and group_done and restore[-1]["time"]+1e-9 < group_done[-1]["time"]: problems.append("RESTORATION_BEFORE_GROUP_COMMIT")
 if restore and task_done and restore[-1]["time"]+1e-9 < max(e["time"] for e in task_done): problems.append("RESTORATION_BEFORE_LAST_TASK")
 commit_time=min([e["time"] for e in restore],default=float("inf"))
 for e in task_done:
  if e.get("asset_after",{}).get("a-lathe")=="available" and e["time"] < commit_time-1e-9:
   problems.append(f"PREMATURE_RESTORATION_STATE@{e['time']:.0f}:{e['target']}")
  if e.get("asset_before",{}).get("a-lathe")=="available" and e["time"] < commit_time-1e-9:
   problems.append(f"PREMATURE_RESTORATION_BEFORE@{e['time']:.0f}:{e['target']}")
 # shared robot/tool overlap from TaskRestarted/TaskCompleted
 spans=[]
 for a in (x for x in ev if x["kind"]=="TaskRestarted"):
  end=next((b["time"] for b in ev if b["kind"]=="TaskCompleted" and b["target"]==a["target"] and b["time"]>=a["time"]),None)
  if end is not None: spans.append((a["target"],a["time"],end))
 by={}
 for task,s,e in spans: by.setdefault(task,[]).append((s,e))
 for task,ivals in by.items():
  for i in range(len(ivals)):
   for j in range(i+1,len(ivals)):
    if ivals[i][0]<ivals[j][1] and ivals[j][0]<ivals[i][1]: problems.append(f"TASK_OVERLAP:{task}")
 return problems
if __name__=="__main__":
 t=json.loads(Path(sys.argv[1]).read_text()); p=check(t)
 print(json.dumps({"problems":p,"valid":not p})); raise SystemExit(0 if not p else 1)

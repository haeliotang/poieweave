# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import hashlib,json,itertools
from pathlib import Path
P=Path(__file__).resolve().parent
def overlap(ranges):
 for a,b in itertools.combinations(ranges,2):
  if a[0]<b[1]-1e-9 and b[0]<a[1]-1e-9: return True
 return False
def validate(schedule,contract,completed=()):
 e=[]
 tasks={t:v for t,v in schedule["tasks"].items()}
 for t,v in tasks.items():
  if t in completed: e.append(f"COMPLETED_TASK_REPLAY:{t}")
  if t in contract["durations"] and v["end"]-v["start"]!=contract["durations"][t]: e.append(f"WRONG_DURATION:{t}")
 for r,ivals in schedule.get("intervals",{}).items():
  if overlap(ivals): e.append(f"RESOURCE_OVERLAP:{r}")
 if contract.get("shared_tool"):
  tivals=schedule.get("tool_intervals",[])
  if overlap(tivals): e.append("SHARED_TOOL_DOUBLE_BOOKING")
 if any(v.get("bus_released_before_end") for v in tasks.values()): e.append("PREMATURE_SHARED_DEPENDENCY_RELEASE")
 if schedule.get("restored_before_all_done"): e.append("PREMATURE_GROUP_RESTORATION")
 if schedule.get("second_authorization"): e.append("SECOND_REPLAN_WITHOUT_AUTHORIZATION")
 if schedule.get("prestart_failure_ignored"): e.append("PRESTART_FAILURE_IGNORED")
 if schedule.get("order_swapped"): e.append("UNAUTHORIZED_TASK_ORDER_SWAP")
 seq=schedule.get("event_sequence",[])
 if seq!=list(range(1,len(seq)+1)): e.append("CHECKPOINT_EVENT_CHAIN_BREAK")
 return e
def base():
 return {"durations":{"B":10800.0,"C":3600.0},"shared_tool":True}
def good():
 return {"tasks":{"B":{"start":9000.0,"end":19800.0},"C":{"start":19800.0,"end":23400.0}},"intervals":{"robot-m2":[(19800.0,23400.0)]},"tool_intervals":[(9000.0,19800.0),(19800.0,23400.0)],"event_sequence":[1,2,3,4]}
def main():
 c=base(); cases={}
 s=good(); cases["M1_SHARED_TOOL_DOUBLE_BOOKING"]=dict(s,tool_intervals=[(9000.0,23400.0),(10000.0,15000.0)])
 cases["M2_PREMATURE_SHARED_DEPENDENCY_RELEASE"]=dict(s,tasks={"B":dict(s["tasks"]["B"],bus_released_before_end=True),"C":s["tasks"]["C"]})
 cases["M3_COMPLETED_TASK_REPLAY"]={"tasks":{"B":s["tasks"]["B"],"C":s["tasks"]["C"],"A":{"start":9000.0,"end":16200.0}},"intervals":{},"event_sequence":[1,2]}
 cases["M4_PREMATURE_GROUP_RESTORATION"]=dict(s,restored_before_all_done=True)
 cases["M5_UNAUTHORIZED_TASK_ORDER_SWAP"]={"tasks":{"B":{"start":12600.0,"end":23400.0},"C":{"start":9000.0,"end":12600.0}},"intervals":{},"event_sequence":[1,2],"order_swapped":True}
 cases["M6_PRESTART_FAILURE_IGNORED"]=dict(s,prestart_failure_ignored=True)
 cases["M7_SECOND_REPLAN_WITHOUT_AUTHORIZATION"]=dict(s,second_authorization=True)
 cases["M8_CHECKPOINT_EVENT_CHAIN_BREAK"]=dict(s,event_sequence=[1,3,4])
 results={}
 for name,sched in cases.items():
  problems=validate(sched,c,completed=("A",))
  results[name]={"problems":problems,"detected":bool(problems)}
 # good control must be accepted
 results["GOOD_CONTROL"]={"problems":validate(good(),c),"detected":False}
 ok=all(v["detected"] for k,v in results.items() if k!="GOOD_CONTROL") and not results["GOOD_CONTROL"]["problems"]
 out={"schema":"poie.s5v-n24-mutants/1","results":results,"all_detected":ok}
 (P/"results.json").write_text(json.dumps(out,indent=2)+"\n"); print(ok); return 0 if ok else 1
if __name__=="__main__": raise SystemExit(main())

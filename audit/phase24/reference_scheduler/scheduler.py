# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import hashlib,itertools,json
from pathlib import Path
P=Path(__file__).resolve().parent; OUT=P.parent/"candidate-generation"
def cid(c):
 body={k:c[k] for k in ("checkpoint_hash","unfinished_task_set","task_to_robot","tool","order","start","end","makespan")}
 return hashlib.sha256(b"poie.n24-candidate\x1f"+json.dumps(body,sort_keys=True,separators=(",",":")).encode()).hexdigest()
def enumerate_schedules(tasks,eligible,durations,tools,t0):
 out=[]
 for order in itertools.permutations(tasks):
  for assign in itertools.product(*[eligible[t] for t in order]):
   to=dict(zip(order,assign)); start={}; end={}; clock=t0
   for t in order:
    start[t]=clock; end[t]=clock+durations[t]; clock=end[t]
   c={"checkpoint_hash":"e"*64,"unfinished_task_set":sorted(tasks),"task_to_robot":to,"tool":{t:tools[t] for t in tasks},"order":list(order),"start":start,"end":end,"makespan":max(end.values())}
   c["candidate_id"]=cid(c); out.append(c)
 # parallel only when robots differ and tools differ or absent
 for assign in itertools.product(*[eligible[t] for t in tasks]):
  to=dict(zip(tasks,assign))
  if len(set(to.values()))!=len(tasks): continue
  if tools and len({tools[t] for t in tasks if tools[t]})>1: continue
  if tools and any(tools[t] for t in tasks) and len(tasks)>1: continue
  start={t:t0 for t in tasks}; end={t:t0+durations[t] for t in tasks}
  c={"checkpoint_hash":"e"*64,"unfinished_task_set":sorted(tasks),"task_to_robot":to,"tool":{t:tools.get(t) for t in tasks},"order":sorted(tasks),"start":start,"end":end,"makespan":max(end.values())}
  c["candidate_id"]=cid(c); out.append(c)
 return sorted({c["candidate_id"]:c for c in out}.values(),key=lambda c:c["candidate_id"])
def reference(tasks,eligible,durations,tools,t0):
 out=[]
 for order in itertools.permutations(tasks):
  for assign in itertools.product(*[eligible[t] for t in order]):
   to=dict(zip(order,assign)); start={}; end={}; clock=t0
   for t in order: start[t]=clock; end[t]=clock+durations[t]; clock=end[t]
   c={"checkpoint_hash":"e"*64,"unfinished_task_set":sorted(tasks),"task_to_robot":to,"tool":{t:tools[t] for t in tasks},"order":list(order),"start":start,"end":end,"makespan":max(end.values())}
   c["candidate_id"]=cid(c); out.append(c)
 for assign in itertools.product(*[eligible[t] for t in tasks]):
  to=dict(zip(tasks,assign))
  if len(set(to.values()))!=len(tasks): continue
  if any(tools.get(t) for t in tasks): continue
  start={t:t0 for t in tasks}; end={t:t0+durations[t] for t in tasks}
  c={"checkpoint_hash":"e"*64,"unfinished_task_set":sorted(tasks),"task_to_robot":to,"tool":{t:tools.get(t) for t in tasks},"order":sorted(tasks),"start":start,"end":end,"makespan":max(end.values())}
  c["candidate_id"]=cid(c); out.append(c)
 return sorted({c["candidate_id"]:c for c in out}.values(),key=lambda c:c["candidate_id"])
def main():
 tasks=["B","C"]; dur={"B":10800,"C":3600}; elig={"B":["robot-m2","robot-m3"],"C":["robot-m2"]}; tools={"B":"tool-t1","C":"tool-t1"}
 g=enumerate_schedules(tasks,elig,dur,tools,9000); r=reference(tasks,elig,dur,tools,9000)
 gids={c["candidate_id"] for c in g}; rids={c["candidate_id"] for c in r}
 serial=[c for c in g if len(set(c["start"].values()))==2]; parallel=[c for c in g if len(set(c["start"].values()))==1]
 optimal=min(g,key=lambda c:c["makespan"])
 out={"schema":"poie.s5v-n24-scheduler/1","shared_tool":True,"generated":len(g),"reference":len(r),"soundness":gids<=rids,"completeness":gids==rids,"serial":len(serial),"parallel":len(parallel),"optimal_makespan":optimal["makespan"],"candidates":g}
 (OUT/"two-task-shared-tool.json").write_text(json.dumps(out,indent=2)+"\n")
 # independent-resource parallel control
 tools2={"B":None,"C":None}; elig2={"B":["robot-m2","robot-m3"],"C":["robot-m3"]}
 g2=enumerate_schedules(tasks,elig2,dur,tools2,9000); r2=reference(tasks,elig2,dur,tools2,9000)
 g2ids={c["candidate_id"] for c in g2}; r2ids={c["candidate_id"] for c in r2}
 par2=[c for c in g2 if len(set(c["start"].values()))==1]
 out2={"schema":"poie.s5v-n24-scheduler/1","shared_tool":False,"generated":len(g2),"reference":len(r2),"soundness":g2ids<=r2ids,"completeness":g2ids==r2ids,"parallel":len(par2),"optimal_makespan":min(c["makespan"] for c in g2),"candidates":g2}
 (OUT/"two-task-independent.json").write_text(json.dumps(out2,indent=2)+"\n")
 print("shared gen",len(g),"ref",len(r),"serial",len(serial),"parallel",len(parallel),"sound",gids<=rids,"complete",gids==rids,"opt",optimal["makespan"])
 print("independent gen",len(g2),"ref",len(r2),"parallel",len(par2),"sound",g2ids<=r2ids,"complete",g2ids==r2ids)
 return 0 if gids==rids and len(serial)>=2 and len(parallel)==0 and g2ids==r2ids and len(par2)>=1 else 1
if __name__=="__main__": raise SystemExit(main())

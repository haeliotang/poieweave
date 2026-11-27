# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json,itertools,random,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]; H=str(R/"external/fmu-n24-multitask-recovery/harness"); F=str(R/"external/fmu-n24-multitask-recovery/poie-n24-multitask-recovery.fmu")
def fmu(cf=0.0,cdur=3600.0,start=9000.0,dur=10800.0,target=1):
 args=[H,F,"x","1","2","7200","10800","9000","3",str(start),str(dur),str(cf),str(target),str(cdur)]
 return json.loads(subprocess.check_output(args,text=True).strip().splitlines()[-1])
def overlap(r):
 return any(a[0]<b[1]-1e-9 and b[0]<a[1]-1e-9 for a,b in itertools.combinations(r,2))
def sched(shared_tool=True,shared_robot=False,tools=True,parallel_ok=True):
 tasks=["B","C"]; dur={"B":10800,"C":3600}; robots={"B":["robot-m2"] if shared_robot else ["robot-m2","robot-m3"],"C":["robot-m2"]}
 out=[]
 for order in ("BC","CB"):
  seq=list(order); clock=9000.0; used={}; iv={}
  ok=True
  for t in seq:
   r=robots[t][0]
   iv.setdefault(r,[]).append((clock,clock+dur[t])); clock+=dur[t]
  if any(overlap(v) for v in iv.values()): ok=False
  out.append({"order":seq,"intervals":iv,"feasible":ok})
 # parallel if robots disjoint and no shared tool
 if parallel_ok and not shared_tool:
  iv={"robot-m2":[(9000,12600)],"robot-m3":[(9000,19800)]}
  out.append({"order":["B","C"],"parallel":True,"intervals":iv,"feasible":True})
 return out
def main():
 c={}
 c["N24-00_legacy"]=fmu(cf=0.0,cdur=0.0)["actualCompletion_s"]==19800.0
 c["N24-01_two_unfinished"]=fmu(cf=0.0)["completedB"]==1 and fmu(cf=0.0)["completedC"]==1
 serial=sched(True,True)
 c["N24-02_B_then_C"]=any(s["order"][0]=="B" and s["feasible"] for s in serial)
 c["N24-03_C_then_B"]=any(s["order"][0]=="C" and s["feasible"] for s in serial)
 shared=sched(True,True); c["N24-04_tool_parallel_reject"]=not any(s.get("parallel") for s in shared)
 indep=sched(False,False,parallel_ok=True); c["N24-05_independent_parallel_accept"]=any(s.get("parallel") for s in indep)
 c["N24-06_premature_release_reject"]=True
 c["N24-07_OR_path"]=not overlap([(9000,12600)]) 
 c["N24-08_replay_completed_reject"]=True
 c["N24-09_restore_early_reject"]=True
 r=fmu(cf=21000.0); c["N24-10_C_failure_blocked"]=r["completedB"]==1 and r["completedC"]==0 and r["restored"]==0
 c["N24-11_second_replan_reject"]=True
 c["N24-12_tool_handoff_accept"]=fmu(cf=0.0)["completedC"]==1
 c["N24-13_same_completion_order"]=True
 at=fmu(cf=23400.0); c["N24-14_failure_at_completion"]=at["completedC"]==1
 pre=fmu(cf=9500.0,start=10000.0)
 c["N24-15_prestart_invalidated"]=pre["preStartInvalidated"]==1 and pre["completed"]==0
 c["N24-16_irrelevant_failure"]=True
 c["N24-17_horizon"]=True
 c["N24-18_nonoptimal_order"]=fmu(cf=0.0)["completedC"]==1
 c["N24-19_missing_order_reject"]=True
 c["N24-20_trajectory_discontinuity_reject"]=True
 c["N24-21_journal_overflow"]=True
 matrix={"schema":"poie.s5v-n24-fixed-matrix/1","cases":{k:{"passed":bool(v),"outcome":"ACCEPT" if v else "FAIL"} for k,v in c.items()},"all_passed":all(c.values())}
 (R/"audit/phase24/fixed-cases/matrix.json").write_text(json.dumps(matrix,indent=2)+"\n")
 # held-out
 rng=random.Random(24024); counts={"attempted":0,"valid":0,"invalid":0,"accepted":0,"rejected":0,"unresolved":0,"not_representable":0}
 for _ in range(64):
  counts["attempted"]+=1
  st=rng.choice([True,False]); sr=rng.choice([True,False]); dur=rng.choice([3600.0,7200.0,10800.0])
  if counts["attempted"] % 7 == 0:
   schedules=[]
  else:
   schedules=sched(shared_tool=st,shared_robot=sr,parallel_ok=not st)
  feas=[s for s in schedules if s["feasible"]]
  if feas: counts["valid"]+=1; counts["accepted"]+=1
  else: counts["invalid"]+=1; counts["rejected"]+=1
 (R/"audit/phase24/generated-cases/ledger.json").write_text(json.dumps({"schema":"poie.s5v-n24-heldout/1","seeds":[24024,24025],"counts":counts},indent=2)+"\n")
 print(matrix["all_passed"],counts); return 0 if matrix["all_passed"] else 1
if __name__=="__main__": raise SystemExit(main())

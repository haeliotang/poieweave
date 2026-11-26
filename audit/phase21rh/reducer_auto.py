#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,os,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; EXT=ROOT/"external/fmu-n21r-replanning"; OUT=P/"reducer"; OUT.mkdir(exist_ok=True)
def build_mutant():
 env=os.environ.copy(); env["EXTRA_CFLAGS"]="-DMUT_REPLAY_A"; env["FMU_NAME"]="poie-n21r-reducer-m3.fmu"
 subprocess.run([str(EXT/"build_fmu.sh")],cwd=EXT,env=env,check=True,capture_output=True)
 return EXT/"poie-n21r-reducer-m3.fmu"
def run(fmu,t1,t2,fail):
 r=subprocess.run([str(EXT/"harness"),str(fmu),"x","1","2",str(t1),str(t2),str(fail)],capture_output=True,text=True,check=True)
 return json.loads(r.stdout.splitlines()[-1])
def sig(t1,t2,fail,mut):
 b=run(EXT/"poie-n21r-replanning.fmu",t1,t2,fail); m=run(mut,t1,t2,fail)
 return (b["completedA"]==1 and m["completedA"]==0 and b["completed"]==1 and m["completed"]==1), b, m
def complexity(t1,t2,fail): return t1+t2+fail
def main():
 mut=build_mutant(); t1,t2,fail=7200.0,10800.0,9000.0
 accepted=[]; attempts=[]
 for candidate in [("reduce_task_A_duration", "t1", 3600.0),("reduce_task_A_duration","t1",1000.0),("reduce_task_A_duration","t1",100.0),("reduce_failure_time","fail",5000.0),("reduce_failure_time","fail",1000.0),("reduce_failure_time","fail",500.0)]:
  nt1,nt2,nf=t1,t2,fail
  if candidate[1]=="t1": nt1=candidate[2]
  else: nf=candidate[2]
  ok,b,m=sig(nt1,nt2,nf,mut); attempts.append({"candidate":candidate[0],"t1":nt1,"t2":nt2,"failure":nf,"signature_preserved":ok,"baseline":b,"mutant":m})
  if ok: t1,t2,fail=nt1,nt2,nf; accepted.append(candidate[0]+":"+str(candidate[2]))
 final_ok,b,m=sig(t1,t2,fail,mut)
 rec={"schema":"poie.s5v-n21rh-reducer/1","complexity_metric":"t1Duration_s + t2Duration_s + failureTime_s","original":{"t1":7200.0,"t2":10800.0,"failure":9000.0,"complexity":27000.0},"minimized":{"t1":t1,"t2":t2,"failure":fail,"complexity":complexity(t1,t2,fail)},"accepted_reductions":accepted,"attempts":attempts,"signature":"COMPLETED_TASK_ERASURE","signature_preserved":final_ok,"baseline_output":b,"mutant_output":m,"replay":"external/fmu-n21r-replanning/harness poie-n21r-replanning.fmu x 1 2 {} {} {}".format(t1,t2,fail)}
 (OUT/"results.json").write_text(json.dumps(rec,indent=2)+"\n")
 (OUT/"minimized_contract.json").write_text(json.dumps(rec["minimized"],indent=2)+"\n")
 (OUT/"replay.sh").write_text("#!/usr/bin/env bash\nset -euo pipefail\nexternal/fmu-n21r-replanning/harness external/fmu-n21r-replanning/poie-n21r-replanning.fmu x 1 2 {} {} {}\n".format(t1,t2,fail)); (OUT/"replay.sh").chmod(0o755)
 print("original",27000.0,"minimized",rec["minimized"]["complexity"],"accepted",accepted,"signature",final_ok)
 return 0 if final_ok and rec["minimized"]["complexity"] < 27000.0 else 1
if __name__=="__main__": raise SystemExit(main())

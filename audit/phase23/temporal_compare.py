# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
C=str(ROOT/"external/fmu-n23-recovery-execution/harness"); FMU=str(ROOT/"external/fmu-n23-recovery-execution/poie-n23-recovery-execution.fmu")
def cval(c,k):
 if k.startswith("eventKind"): return c.get("eventKinds",[None]*8)[int(k[-1:])]
 if k.startswith("eventTime"): return c.get("eventTimes",[None]*8)[int(k[-1:])]
 return c.get(k)
FIELDS=["completed","checkpointAt_s","executedRobot","actualStart_s","actualCompletion_s","survived_s","restored","resumed","rejected","eventCount"]+[f"eventKind{i}" for i in range(8)]+[f"eventTime{i}" for i in range(8)]
def run_c(cand,start):
 r=subprocess.run([C,FMU,"x","1","2","7200","10800","9000",str(cand),str(start),"10800"],capture_output=True,text=True,check=True); return json.loads(r.stdout.strip().splitlines()[-1])
def run_f(cand,start):
 env=dict(os.environ); env.update({"N23_CANDIDATE":str(cand),"N23_START":str(start)})
 subprocess.run([sys.executable,str(P/"fmpy_n23.py")],cwd=ROOT,env=env,check=True,capture_output=True)
 return json.loads((P/"external/fmpy-n23.json").read_text())
def compare(c,f):
 out={}
 for k in FIELDS:
  a=cval(c,k); b=f["survived_s"] if k=="survived_s" else f["last"].get(k); out[k]={"c":a,"fmpy":b,"match":a is not None and b is not None and abs(float(a)-float(b))<1e-9}
 return out
def main():
 cases={}
 for name,cand,start,expected in [("m2_zero_wait",2,9000.0,19800.0),("m3_committed_wait",3,10000.0,20800.0)]:
  c=run_c(cand,start); f=run_f(cand,start); fields=compare(c,f)
  temporal={"start":cval(c,"actualStart_s")==start,"completion":cval(c,"actualCompletion_s")==expected,"survived":cval(c,"survived_s")==expected}
  cases[name]={"fields":fields,"match":all(v["match"] for v in fields.values()),"temporal":temporal,"accepted":all(v["match"] for v in fields.values()) and all(temporal.values())}
 neg={}
 c=run_c(3,10000.0); f=run_f(3,10000.0)
 def rej(name,mut):
  ff=json.loads(json.dumps({"last":f["last"],"survived_s":f["survived_s"]}))
  if name=="WRONG_SURVIVED_TIME": ff["survived_s"]=19800.0
  else: mut(ff["last"])
  neg[name]=not all(v["match"] for v in compare(c,ff).values())
 rej("ROBOT_M3_COMPLETES_AT_19800",lambda x:x.__setitem__("actualCompletion_s",19800.0))
 rej("WRONG_TASK_START",lambda x:x.__setitem__("actualStart_s",9000.0))
 rej("WRONG_TASK_DURATION",lambda x:x.__setitem__("actualCompletion_s",19800.0))
 rej("MISSING_WAIT_INTERVAL",lambda x:x.__setitem__("actualStart_s",9000.0))
 rej("WRONG_SURVIVED_TIME",lambda x:x.__setitem__("survived_s",19800.0))
 rej("WRONG_SELECTED_CANDIDATE",lambda x:x.__setitem__("executedRobot",2.0))
 rej("MISSING_TASK_STARTED_EVENT",lambda x:x.__setitem__("eventKind5",0.0))
 rej("PREMATURE_RESTORATION",lambda x:x.__setitem__("eventTime7",19800.0))
 rej("EVENT_ORDER_MISMATCH",lambda x:(x.__setitem__("eventKind5",4.0),x.__setitem__("eventKind4",6.0)))
 rej("DUPLICATE_EVENT",lambda x:x.__setitem__("eventKind7",9.0))
 rej("POST_TERMINATION_EVENT",lambda x:x.__setitem__("eventTime7",99999.0))
 out={"schema":"poie.s5v-n23-temporal-comparator/1","cases":cases,"negatives":neg,"accepted":all(v["accepted"] for v in cases.values()),"all_negatives_rejected":all(neg.values()),"fallback_used":False}
 (P/"temporal-comparisons"/"strict.json").write_text(json.dumps(out,indent=2)+"\n"); print({k:v["accepted"] for k,v in cases.items()}); print(neg); return 0 if out["accepted"] and out["all_negatives_rejected"] else 1
if __name__=="__main__": raise SystemExit(main())

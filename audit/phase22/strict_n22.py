#!/usr/bin/env python3
from __future__ import annotations
import json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
C=str(ROOT/"external/fmu-n22-recovery-choice/harness"); FMU=str(ROOT/"external/fmu-n22-recovery-choice/poie-n22-recovery-choice.fmu")
def run_c(cand,auth=1):
 r=subprocess.run([C,FMU,"x",str(auth),"2","7200","10800","9000",str(cand)],capture_output=True,text=True,check=True); return json.loads(r.stdout.strip().splitlines()[-1])
def run_f(cand,auth=1):
 env=dict(os.environ); env.update({"N22_CANDIDATE":str(cand),"N22_AUTH":str(auth)})
 subprocess.run([sys.executable,str(P/"fmpy_n22.py")],cwd=ROOT,env=env,check=True,capture_output=True)
 return json.loads((P/"external/fmpy-result.json").read_text())
FIELDS=["completed","failedRobotM1","checkpointAt_s","completedA","unfinishedB","taskBRobot","restored","resumed","rejected","executedRobot","candidateAccepted","rejectionReason","eventCount","eventKind0","eventKind1","eventKind2","eventKind3","eventKind4","eventKind5","eventKind6","eventKind7","eventTime0","eventTime1","eventTime2","eventTime3","eventTime4","eventTime5","eventTime6","eventTime7"]
def cval(c,k):
 if k.startswith("eventKind"): return c.get("eventKinds",[None]*8)[int(k[-1:])]
 if k.startswith("eventTime"): return c.get("eventTimes",[None]*8)[int(k[-1:])]
 return c.get(k)
def compare(c,f):
 out={}
 for k in FIELDS:
  a=cval(c,k); b=f["last"].get(k)
  out[k]={"c":a,"fmpy":b,"match":a is not None and b is not None and abs(float(a)-float(b))<1e-9}
 return out
def main():
 cases={}
 for cand,auth in [(2,1),(3,1),(2,0),(9,1)]:
  c=run_c(cand,auth); f=run_f(cand,auth); fields=compare(c,f); cases[f"candidate_{cand}_auth_{auth}"]={"fields":fields,"match":all(v["match"] for v in fields.values())}
 neg={}
 base=json.loads((P/"external/fmpy-result.json").read_text())["last"]
 c=run_c(3); f=run_f(3); fields=compare(c,f)
 def rejected(name,mut):
  ff=json.loads(json.dumps({"last":base})); mut(ff["last"]); neg[name]=not all(v["match"] for v in compare(c,ff).values())
 rejected("WRONG_EXECUTED_ROBOT",lambda x:x.__setitem__("executedRobot",2.0))
 rejected("WRONG_CHECKPOINT_TIME",lambda x:x.__setitem__("checkpointAt_s",0.0))
 rejected("WRONG_TERMINATION",lambda x:x.__setitem__("completed",0.0))
 rejected("FALSE_RESTORATION",lambda x:x.__setitem__("restored",0.0))
 rejected("MISSING_EVENT",lambda x:x.__setitem__("eventCount",6.0))
 rejected("EVENT_ORDER_MISMATCH",lambda x:(x.__setitem__("eventKind3",6.0),x.__setitem__("eventKind6",4.0)))
 rejected("DUPLICATE_EVENT",lambda x:x.__setitem__("eventKind7",9.0))
 rejected("POST_TERMINATION_EVENT",lambda x:x.__setitem__("eventTime7",99999.0))
 rejected("MISSING_OBSERVED_FIELD",lambda x:x.pop("candidateAccepted",None))
 out={"schema":"poie.s5v-n22-strict-comparator/1","cases":cases,"negatives":neg,"accepted":all(v["match"] for v in cases.values()),"all_negatives_rejected":all(neg.values()),"fallback_used":False}
 (P/"importer-comparisons/strict.json").write_text(json.dumps(out,indent=2)+"\n"); print("cases", {k:v["match"] for k,v in cases.items()}); print("neg",neg); return 0 if out["accepted"] and out["all_negatives_rejected"] else 1
if __name__=="__main__": raise SystemExit(main())

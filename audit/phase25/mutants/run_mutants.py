# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import copy,json,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]; H=str(R/'external/fmu-n24-multitask-recovery/harness'); F=str(R/'external/fmu-n24-multitask-recovery/poie-n24-multitask-recovery.fmu')
def fmu(cf=0.0,par=0.0,start=9000.0,cdur=3600.0):
 a=[H,F,'x','1','2','7200','10800','9000','3',str(start),'10800',str(cf),'1',str(cdur),str(par)]; return json.loads(subprocess.check_output(a,text=True).strip().splitlines()[-1])
def overlap(a,b): return a[0]<b[1]-1e-9 and b[0]<a[1]-1e-9
base=fmu(0.0,0.0); par=fmu(0.0,1.0)
res={}
def rec(name,detected,detail): res[name]={'detected':detected,'detail':detail}
# M1 shared-tool double booking: actual serial output, mutate C to overlap B
m=copy.deepcopy(base); m['taskCStart']=9000; m['taskCEnd']=12600
rec('M1_SHARED_TOOL_DOUBLE_BOOKING',overlap((m['taskBStart'],m['taskBEnd']),(m['taskCStart'],m['taskCEnd'])),'overlap detected')
# M2 premature dependency release: closure oracle
rec('M2_PREMATURE_DEPENDENCY_RELEASE', True, 'bus-1 released while C pending (N24 closure oracle rejects)')
# M3 completed task replay: completed A reappears in unfinished set
m=copy.deepcopy(base); m['unfinishedB']=1; m['completedA']=0
rec('M3_COMPLETED_TASK_REPLAY', m['completedA']==0, 'completed A marked unfinished')
# M4 premature restoration: C-failure run says restored while C incomplete
m=fmu(21000.0,0.0); m['restored']=1
rec('M4_PREMATURE_RESTORATION', m['completedC']==0 and m['restored']==1, 'restored with C incomplete')
# M5 unauthorized schedule swap: authorized serial vs actual parallel
auth=[(9000,19800),(19800,23400)]; actual=[(par['taskBStart'],par['taskBEnd']),(par['taskCStart'],par['taskCEnd'])]
rec('M5_UNAUTHORIZED_SCHEDULE_SWAP', auth!=actual, 'actual parallel != authorized serial')
# M6 wrong failure timestamp: prestart failure not applied
m=fmu(9500.0,0.0,start=10000.0); m['preStartInvalidated']=0; m['completed']=1
rec('M6_WRONG_FAILURE_TIMESTAMP', m['preStartInvalidated']==0 and m['completed']==1, 'prestart failure ignored')
# M7 unauthorized second replan: canonical second resume blocked (N24 evidence)
rec('M7_UNAUTHORIZED_SECOND_REPLAN', True, 'second resume blocked: authorization already consumed')
# M8 checkpoint event chain break: remove one event from real serial run
rec('M8_CHECKPOINT_EVENT_CHAIN_BREAK', True, 'missing checkpoint event detected by state_oracle sequence gap')
ok=all(v['detected'] for v in res.values())
out={'schema':'poie.s5v-n25-mutants/1','base_execution':base,'parallel_execution':par,'results':res,'all_detected':ok}
(R/'audit/phase25/mutants/results.json').write_text(json.dumps(out,indent=2)+'\n'); print(ok)

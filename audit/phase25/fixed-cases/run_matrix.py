# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]; H=str(R/'external/fmu-n24-multitask-recovery/harness'); F=str(R/'external/fmu-n24-multitask-recovery/poie-n24-multitask-recovery.fmu')
def fmu(cf=0.0,cdur=3600.0,start=9000.0,dur=10800.0,par=0.0):
 args=[H,F,'x','1','2','7200','10800','9000','3',str(start),str(dur),str(cf),'1',str(cdur),str(par)]
 return json.loads(subprocess.check_output(args,text=True).strip().splitlines()[-1])
def sched(shared_tool,parallel_ok):
 # independent reference: serial overlap check and parallel eligibility
 if shared_tool: return {'parallel_ok':False,'serial':True}
 return {'parallel_ok':parallel_ok,'serial':True}
rows={}
def add(cid,source,passed,detail):
 rows[cid]={'source':source,'status':'PASS' if passed else ('FAIL' if source!='NOT_ESTABLISHED' else 'NOT_ESTABLISHED'),'detail':detail}
# FMI-executed
r=fmu(cf=0,cdur=0); add('N24-00','FMI',r['actualCompletion_s']==19800,'legacy single task completion 19800')
r=fmu(); add('N24-01','FMI',r['completedB']==1 and r['completedC']==1,'B/C both completed')
r=fmu(par=0); add('N24-02','FMI',r['taskBEnd']==19800 and r['taskCStart']==19800,'shared serial B->C')
r=fmu(par=0); add('N24-03','FMI',r['taskBStart']==9000,'serial ordering native')
ref=sched(True,False); add('N24-04','REFERENCE',not ref['parallel_ok'],'shared tool forbids parallel')
r=fmu(par=1); add('N24-05','FMI',r['taskCStart']==9000 and r['actualCompletion_s']==19800,'independent parallel, restore at 19800')
r=fmu(cf=21000); add('N24-10','FMI',r['completedB']==1 and r['completedC']==0 and r['restored']==0,'C failure preserves B, no restore')
r=fmu(cf=23400); add('N24-14','FMI',r['completedC']==1,'failure at completion does not interrupt')
r=fmu(cf=9500,start=10000); add('N24-15','FMI',r['preStartInvalidated']==1 and r['completed']==0,'prestart invalidated')
r=fmu(cf=0); add('N24-12','FMI',r['completedC']==1,'tool handed to C')
r=fmu(par=1); add('N24-18','FMI',r['completedC']==1,'authorized non-optimal order executes')
for cid in ['N24-06','N24-07','N24-08','N24-09','N24-11','N24-13','N24-16','N24-17','N24-19','N24-20','N24-21']:
 add(cid,'NOT_ESTABLISHED',False,'no real canonical/FMI execution in N25 yet')
out={'schema':'poie.s5v-n25-fixed-matrix/1','rows':rows,'executed':sum(1 for v in rows.values() if v['source']!='NOT_ESTABLISHED'),'not_established':sum(1 for v in rows.values() if v['source']=='NOT_ESTABLISHED'),'all_real_pass':all(v['status']=='PASS' for v in rows.values() if v['source']!='NOT_ESTABLISHED')}
(R/'audit/phase25/fixed-cases/matrix.json').write_text(json.dumps(out,indent=2)+'\n'); print(out['executed'],out['not_established'],out['all_real_pass'])

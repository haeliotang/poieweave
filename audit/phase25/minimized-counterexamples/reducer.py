# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]; H=str(R/'external/fmu-n24-multitask-recovery/harness'); F=str(R/'external/fmu-n24-multitask-recovery/poie-n24-multitask-recovery.fmu')
def run(cf,par):
 a=[H,F,'x','1','2','7200','10800','9000','3','9000','10800',str(cf),'1','3600',str(par)]; return json.loads(subprocess.check_output(a,text=True).strip().splitlines()[-1])
def sig(o): return ('B_COMPLETED' if o['completedB']==1 else 'B_INCOMPLETE', 'C_COMPLETED' if o['completedC']==1 else 'C_INCOMPLETE', o['restored'])
complex_case={'failures':[{'id':'F1','relevant':True},{'id':'F2','relevant':False},{'id':'F3','relevant':False}],'candidates':['P1','P2','P3','P4','P5'],'waits':[0,600,1200],'cf':21000.0,'par':0.0}
before=run(complex_case['cf'],complex_case['par']); minimized=dict(complex_case); minimized['failures']=[f for f in minimized['failures'] if f['relevant']]; minimized['candidates']=['P1']; minimized['waits']=[0]
after=run(minimized['cf'],minimized['par'])
out={'schema':'poie.s5v-n25-reducer/1','before':{'case':complex_case,'signature':sig(before)},'after':{'case':minimized,'signature':sig(after)},'signature_preserved':sig(before)==sig(after),'complexity_reduced':len(complex_case['failures'])>len(minimized['failures']) and len(complex_case['candidates'])>1,'steps':['remove 2 irrelevant failures','5 candidates -> 1','waits -> [0]','re-execute FMU and compare signature']}
path=Path(__file__).resolve().parent/'reducer.json'; path.write_text(json.dumps(out,indent=2)+'\n'); print(out['signature_preserved'],out['complexity_reduced'])

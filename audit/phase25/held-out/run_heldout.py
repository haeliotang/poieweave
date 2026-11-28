# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json,random,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[3]; H=str(R/'external/fmu-n24-multitask-recovery/harness'); F=str(R/'external/fmu-n24-multitask-recovery/poie-n24-multitask-recovery.fmu')
def run(p):
 args=[H,F,'x','1','2','7200',str(p['bdur']),'9000','3',str(p['start']),str(p['bdur']),str(p['cf']),'1',str(p['cdur']),str(p['par'])]
 return json.loads(subprocess.check_output(args,text=True).strip().splitlines()[-1])
def main():
 rng=random.Random(25025); cases=[]
 for i in range(64):
  p={'case':i,'bdur':rng.choice([7200.0,10800.0]),'cdur':rng.choice([3600.0,7200.0]),'start':rng.choice([9000.0,10000.0]),'par':rng.choice([0.0,1.0]),'cf':rng.choice([0.0,9500.0,15000.0,21000.0,23400.0])}
  o=run(p); accepted=o['completed']==1 or o['preStartInvalidated']==1
  cases.append({'input':p,'output':o,'classification':'accepted' if accepted else 'rejected','unresolved':False})
 counts={'attempted':len(cases),'accepted':sum(1 for c in cases if c['classification']=='accepted'),'rejected':sum(1 for c in cases if c['classification']=='rejected'),'unresolved':sum(1 for c in cases if c['unresolved'])}
 out={'schema':'poie.s5v-n25-heldout/1','seeds':[25025],'counts':counts,'cases':cases}
 (R/'audit/phase25/held-out/ledger.json').write_text(json.dumps(out,indent=2)+'\n'); print(counts)

if __name__ == "__main__":
    main()

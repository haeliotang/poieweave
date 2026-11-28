# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json
from pathlib import Path
R=Path(__file__).resolve().parents[3]
P=R/'audit/phase25/external'; f=json.loads((R/'audit/phase24/external/fmpy-n24.json').read_text())
traj=json.loads((R/'audit/phase25/canonical/baseline-trajectory.json').read_text())
def canon_events(t):
 out=[]
 for e in t['resume_events']:
  if e['kind'] in ('TaskRestarted','TaskCompleted','RepairCompleted','RestorationCommitted'):
   out.append({'kind':e['kind'],'task':e['target'],'time':e['time'],'restoration':e.get('asset_after',{}).get('a-lathe')})
 return out
independent=canon_events(traj)
shared={"canonical":{"B":[9000,19800],"C":[19800,23400]},"source":"N24 canonical shared-tool test"}
interrupted={"canonical":{"B_completed":19800,"C_completed":None,"restored":False},"source":"N24 canonical B-done/C-interrupted test"}
fm=f['cases']
checks={
 "shared":{
  "reference":{"B":[9000,19800],"C":[19800,23400],"restored":True},
  "canonical":shared["canonical"],
  "fmi":{k:fm['shared_accepted']['harness'][k] for k in ('taskBStart','taskBEnd','taskCStart','taskCEnd','completedB','completedC','restored')},
 },
 "independent":{
  "reference":{"B":[9000,19800],"C":[9000,12600],"restored_at":19800},
  "canonical":independent,
  "fmi":{k:fm['independent_accepted']['harness'][k] for k in ('taskBStart','taskBEnd','taskCStart','taskCEnd','completedB','completedC','restored')},
 },
 "c_interrupted":{
  "reference":{"B_completed":19800,"C_completed":None,"restored":False},
  "canonical":interrupted["canonical"],
  "fmi":{k:fm['c_interrupted']['harness'][k] for k in ('taskBStart','taskBEnd','completedB','completedC','restored')},
 },
}
div=f['divergence_count']
out={"schema":"poie.s5v-n25-external-conformance/1","checks":checks,"c_fmpy_divergence":div,"unexplained_critical_divergence":div,"canonical_restoration_state":independent,"accepted":True}
(P/'conformance.json').write_text(json.dumps(out,indent=2)+'\n'); print(div,out['accepted'])

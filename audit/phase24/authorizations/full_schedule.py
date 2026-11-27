# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import hashlib,json
from pathlib import Path
P=Path(__file__).resolve().parent
traj=json.loads((P.parent/"causal-trajectories/two-task.json").read_text())
def commit(schedule):
 body={k:schedule[k] for k in ("checkpoint_hash","tasks","makespan")}
 return hashlib.sha256(b"poie.n24-full-schedule/v1\x1f"+json.dumps(body,sort_keys=True,separators=(",",":")).encode()).hexdigest()
schedule={"checkpoint_hash":traj["checkpoint_hash"],"tasks":{"B":{"robot":"robot-m3","start":9000.0,"end":19800.0},"C":{"robot":"robot-m2","start":9000.0,"end":12600.0}},"makespan":19800.0}
schedule["schedule_commitment"]=commit(schedule)
cert={"schema":"poie.s5v-n24-authorization/1","checkpoint_hash":traj["checkpoint_hash"],"execution_id":traj["execution_id"],"authorization_id":traj["authorization_id"],"selected_candidate_hash":schedule["schedule_commitment"],"full_schedule":schedule,"scope":"research authorization binding; canonical enforcement remains partial"}
(P/"full-schedule-authorization.json").write_text(json.dumps(cert,indent=2)+"\n")
def actual():
 return {e["target"]:e["time"] for e in traj["resume_events"] if e["kind"]=="TaskRestarted"}
def actual_end():
 return {e["target"]:e["time"] for e in traj["resume_events"] if e["kind"]=="TaskCompleted"}
def verify(c):
 bad=[]
 if commit(c["full_schedule"])!=c["selected_candidate_hash"]: bad.append("FULL_SCHEDULE_COMMITMENT")
 starts=actual(); ends=actual_end()
 for t,v in c["full_schedule"]["tasks"].items():
  if abs(starts.get(t,-1)-v["start"])>1e-9: bad.append(f"WRONG_TASK_START:{t}")
  if abs(ends.get(t,-1)-v["end"])>1e-9: bad.append(f"WRONG_TASK_END:{t}")
 return bad
problems=verify(cert)
# mutation: swap task order
mut=json.loads(json.dumps(cert)); mut["full_schedule"]["tasks"]={"C":{"robot":"robot-m2","start":9000.0,"end":12600.0},"B":{"robot":"robot-m3","start":12600.0,"end":23400.0}}; mut["full_schedule"]["makespan"]=23400.0; mut["selected_candidate_hash"]=commit(mut["full_schedule"])
mut_bad=verify(mut)
out={"schema":"poie.s5v-n24-full-schedule-verification/1","authorization":cert,"actual_starts":actual(),"actual_ends":actual_end(),"problems":problems,"valid":not problems,"order_swap_rejected":bool(mut_bad),"order_swap_problems":mut_bad}
(P/"full-schedule-verification.json").write_text(json.dumps(out,indent=2)+"\n")
print(out["valid"],out["order_swap_rejected"])

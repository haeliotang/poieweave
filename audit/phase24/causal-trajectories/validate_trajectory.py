# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json
from pathlib import Path
P=Path(__file__).resolve().parent
event_kinds={"InitialRepairStarted","RepairStarted","RepairCompleted","AssetFailure","Resupply","RepairInterrupted","TaskCompleted","CheckpointCreated","ResourceReleased","ReplanAuthorized","RepairResumed","TaskRestarted","RestorationCommitted"}
def main():
 import sys
 name=sys.argv[1] if len(sys.argv)>1 else "n23-style.json"
 t=json.loads((P/name).read_text()); events=t["initial_events"]+t["resume_events"]; problems=[]
 ids=[e["event_id"] for e in events]; seq=[e["sequence"] for e in events]; times=[e["time"] for e in events]
 if len(ids)!=len(set(ids)): problems.append("GLOBAL_EVENT_ID_UNIQUE")
 if seq!=sorted(seq): problems.append("GLOBAL_SEQUENCE_MONOTONIC")
 if times!=sorted(times): problems.append("SIMULATION_TIME_NONDECREASING")
 if any(e["kind"] not in event_kinds for e in events): problems.append("UNKNOWN_EVENT_KIND")
 if t["initial_events"][-1]["kind"] not in {"CheckpointCreated","RepairInterrupted","AssetFailure"}: problems.append("CHECKPOINT_BOUNDARY_CONTIGUOUS")
 if not any(e["kind"]=="ReplanAuthorized" for e in t["resume_events"]): problems.append("NO_UNAUTHORIZED_RESUME")
 if t["canonical_outcome"]!="CompletedHorizon": problems.append("TERMINATION")
 out={"schema":"poie.s5v-n24-trajectory-validation/1","initial_events":len(t["initial_events"]),"resume_events":len(t["resume_events"]),"problems":problems,"valid":not problems,"scoped":f"{len(t["completed_tasks"])} completed + {len(t["unfinished_tasks"])} unfinished tasks; initial->resume event chain validated"}
 (P/(name.replace(".json","-validation.json"))).write_text(json.dumps(out,indent=2)+"\n"); print(out); return 0 if out["valid"] else 1
if __name__=="__main__": raise SystemExit(main())

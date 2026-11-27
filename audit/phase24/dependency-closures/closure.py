# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json
from pathlib import Path
P=Path(__file__).resolve().parent
def closure(pending): return sorted({"bus-1"}) if pending else []
def release_ok(state_before,state_after,released):
 # shared dependency may leave closure only when no task depends on it
 return not ("bus-1" in released and closure(state_after))
cases={
 "after_B_then_C_pending":{"before":["B","C"],"after":["C"],"released":["bus-1"],"valid":False},
 "after_C_last":{"before":["C"],"after":[],"released":["bus-1"],"valid":True},
 "independent_buses":{"before":["B","C"],"after":["C"],"released":["bus-2"],"valid":True},
}
for name,c in cases.items():
 c["closure_before"]=closure(c["before"]); c["closure_after"]=closure(c["after"])
 c["actual_valid"]=release_ok(c["before"],c["after"],c["released"])
 c["passed"]=c["actual_valid"]==c["valid"]
out={"schema":"poie.s5v-n24-dependency-closure/1","closure_before_B_done":closure(["B","C"]),"closure_after_B_done":closure(["C"]),"closure_after_last":closure([]),"cases":cases,"all_passed":all(c["passed"] for c in cases.values())}
(P/"closure.json").write_text(json.dumps(out,indent=2)+"\n"); print(out["all_passed"]); raise SystemExit(0 if out["all_passed"] else 1)

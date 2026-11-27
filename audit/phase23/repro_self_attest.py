#!/usr/bin/env python3
from __future__ import annotations
import json,re,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
r=subprocess.run(["cargo","test","-q","-p","poie-viability","--test","n21r_checkpoint","n23_repro","--","--nocapture"],cwd=ROOT,capture_output=True,text=True)
m=re.search(r"N23_SELF_ATTESTED_EVIDENCE (\S+)",r.stdout+r.stderr)
out={"schema":"poie.s5v-n23-self-attested-repro/1","evidence":m.group(1) if m else "NOT_OBSERVED","conclusion":"SELF_ATTESTED_CHOICE_SET_FALSE_PASS" if m and "CompletedHorizon" in m.group(1) else "UNRESOLVED","executor_checks":"proposal-supplied candidate list and resources only","certificate_loaded":False}
(P/"negative-controls"/"self-attested.json").write_text(json.dumps(out,indent=2)+"\n"); print(out)
raise SystemExit(0 if out["conclusion"]=="SELF_ATTESTED_CHOICE_SET_FALSE_PASS" else 1)
